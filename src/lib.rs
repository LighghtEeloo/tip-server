mod typst_library;

use comemo::{Track, Tracked, Validate};
use std::path::PathBuf;
use typst::diag::SourceResult;
use typst::engine::{Engine, Route};
use typst::eval::Tracer;
use typst::foundations::{Content, Label, Selector, Smart, StyleChain};
use typst::introspection::{Introspector, Locator};
use typst::layout::{Abs, LayoutRoot};
use typst::model::Document;
use typst::World;
use typst_library::TypstWrapperWorld;

pub struct LabeledTypstWorld {
    world: TypstWrapperWorld,
    labels: Vec<String>,
    dest: PathBuf,
}

impl LabeledTypstWorld {
    pub fn new(root: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            world: TypstWrapperWorld::new(root.into(), source.into()),
            labels: Vec::new(),
            dest: PathBuf::from("_out"),
        }
    }

    pub fn with(mut self, label: impl Into<String>) -> Self {
        self.labels.push(label.into());
        self
    }
    pub fn render(&self) {
        let world = <dyn World as Track>::track(&self.world);
        let mut tracer = Tracer::default();
        let module = {
            // evaluate the source file into a module
            typst::eval::eval(
                world,
                Route::default().track(),
                tracer.track_mut(),
                &world.main(),
            )
            .unwrap()
        };
        let content = module.content();

        // whole
        {
            let doc = typeset(world, &mut tracer, &content).unwrap();
            let pdf = typst_pdf::pdf(&doc, Smart::Auto, None);
            std::fs::write(self.dest.join("whole.pdf"), pdf).expect("Error writing PDF.");
        }

        // not an ideal logic
        let targets = self
            .labels
            .iter()
            .map(|label| {
                content
                    .query(Selector::Label(Label::new(label.as_str())))
                    .into_iter()
                    .map(|tar| (label.to_owned(), tar))
            })
            .flatten()
            .collect::<Vec<_>>();
        let docs = targets
            .into_iter()
            .map(|(label, tar)| {
                let doc = typeset(world, &mut tracer, &tar).unwrap();
                (label, doc)
            })
            .collect::<Vec<_>>();

        for (label, doc) in docs {
            // Output to pdf and svg
            let pdf = typst_pdf::pdf(&doc, Smart::Auto, None);
            std::fs::write(self.dest.join(format!("{}.pdf", label)), pdf)
                .expect("Error writing PDF.");

            let svg = typst_svg::svg_merged(&doc, Abs::pt(2.));
            std::fs::write(self.dest.join(format!("{}.svg", label)), svg)
                .expect("Error writing SVG.");
        }
    }
}

/// Relayout until introspection converges.
fn typeset(
    world: Tracked<dyn World + '_>,
    tracer: &mut Tracer,
    content: &Content,
) -> SourceResult<Document> {
    let library = world.library();
    let styles = StyleChain::new(&library.styles);

    let mut iter = 0;
    let mut document = Document::default();

    // Relayout until all introspections stabilize.
    // If that doesn't happen within five attempts, we give up.
    loop {
        // Clear delayed errors.
        tracer.delayed();

        let constraint = <Introspector as Validate>::Constraint::new();
        let mut locator = Locator::new();
        let mut engine = Engine {
            world,
            route: Route::default(),
            tracer: tracer.track_mut(),
            locator: &mut locator,
            introspector: document.introspector.track_with(&constraint),
        };

        // Layout!
        document = content.layout_root(&mut engine, styles)?;
        document.introspector.rebuild(&document.pages);
        iter += 1;

        if iter >= 5 {
            break;
        }
    }

    // Promote delayed errors.
    let delayed = tracer.delayed();
    if !delayed.is_empty() {
        return Err(delayed);
    }

    Ok(document)
}

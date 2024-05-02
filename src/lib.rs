mod typst_library;
use comemo::{Track, Tracked, Validate};
use typst::diag::SourceResult;
use typst::engine::{Engine, Route};
use typst::eval::Tracer;
use typst::foundations::{Content, Label, Selector, Smart, StyleChain};
use typst::introspection::{Introspector, Locator};
use typst::layout::LayoutRoot;
use typst::model::Document;
use typst::World;
use typst_library::TypstWrapperWorld;

const TYPST_SAMPLE: &'static str = r##"
#set math.equation(numbering: "(1)")
#{
  let aba = math.op("aba")
  [$
    aba(1)
  $ <aba1>
  ]
};

#let aba = math.op("bab")
$
  integral f d aba(2)
$ <aba2>

"##;

pub fn doc_content() {
    let world = TypstWrapperWorld::new("./".to_string(), TYPST_SAMPLE.to_string());
    let mut tracer = Tracer::default();

    // original
    {
        let document = typst::compile(&world, &mut tracer).expect("Error compiling typst.");

        // Output to pdf and svg
        let pdf = typst_pdf::pdf(&document, Smart::Auto, None);
        std::fs::write("_out/original.pdf", pdf).expect("Error writing PDF.");
    }

    let world = <dyn World as Track>::track(&world);
    let module = {
        // Try to evaluate the source file into a module.
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
        std::fs::write(format!("_out/whole.pdf",), pdf).expect("Error writing PDF.");
    }

    let selector = Selector::Label(Label::new("aba2"));
    // not an ideal logic
    let targets = content.query(selector);
    let docs = targets
        .into_iter()
        .map(|tar| typeset(world, &mut tracer, &tar).unwrap())
        .collect::<Vec<_>>();

    for doc in docs {
        // Output to pdf and svg
        let pdf = typst_pdf::pdf(&doc, Smart::Auto, None);
        std::fs::write(format!("_out/labeled.pdf",), pdf).expect("Error writing PDF.");
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

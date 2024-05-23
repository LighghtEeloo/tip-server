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

$ E = M C ^ 2 $ <einsteins_equation>

$ partial $ <p>

$ multimap $

"##;

fn main() {
    tip_server::LabeledTypstWorld::new("./", TYPST_SAMPLE).with("aba1").with("p").render()
}
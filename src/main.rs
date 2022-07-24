use std::collections::hash_map::HashMap;
use toml::value::Value;
fn create_dirs(name: &str) {
    std::fs::create_dir_all(format!("{name}/lua/{name}")).expect("Error while write dir");
    std::fs::create_dir_all(format!("{name}/colors")).expect("Error while write dir");
}

#[derive(Default, Debug, Clone)]
struct Palette {
    inner: HashMap<String, String>,
}

impl Palette {
    fn get(&self, k: &String) -> Option<&String> {
        self.inner.get(k)
    }
    fn insert(&mut self, k: String, v: String) {
        self.inner.insert(k, v);
    }
    fn set(temp: &Value) -> Self {
        let mut palette_colors = Palette::default();
        let palette = temp.get("palette");
        for (name, color) in palette
            .expect("palette must be set for now")
            .as_table()
            .expect("Value not a table")
            .iter()
        {
            palette_colors.insert(
                String::from(name),
                color.as_str().expect("Expect string").to_string(),
            );
        }
        palette_colors
    }
}

#[derive(Debug, Clone, Default)]
struct Highlights {
    inner: Vec<Highlight>,
}

impl Highlights {
    fn push(&mut self, hl: Highlight) {
        self.inner.push(hl);
    }
    fn set(temp: &Value, palette: &Palette) -> Self {
        let mut highlights = Highlights::default();
        let all_highlights = temp.get("highlights").expect("Cant find highlights");
        let keys = all_highlights
            .as_table()
            .unwrap()
            .keys()
            .map(|k| k.to_string())
            .collect::<Vec<String>>();
        let vals = all_highlights
            .as_table()
            .unwrap()
            .values()
            .collect::<Vec<_>>();
        let mut hl_defs = Vec::<HighlightDefinition>::new();
        for v in vals.iter() {
            hl_defs.push(HighlightDefinition::from_map(
                v.as_table().unwrap(),
                palette,
            ));
        }
        for (k, g) in keys.iter().zip(hl_defs) {
            highlights.push(Highlight::new(k.to_string(), g));
        }
        highlights
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Highlight {
    name: String,
    val: HighlightDefinition,
}

impl Highlight {
    fn new(name: String, val: HighlightDefinition) -> Self {
        Self { name, val }
    }
    fn write_string_lua(&self) -> String {
        let r = if self.val.bg.is_some() {
            let s = format!(
                "vim.api.nvim_set_hl(0,'{name}',{{ fg = '{fg}',bg = '{bg}',bold = {bold},italic = {italic},undercurl = {undercurl},underline = {underline},strikethrough = {strikethrough},reverse = {reverse},nocombine = {nocombine}}})",
                name = self.name,
                fg = self.val.fg,
                bg = self.val.bg.clone().unwrap(),
                bold = self.val.bold.unwrap_or(false),
                italic = self.val.italic.unwrap_or(false),
                undercurl = self.val.undercurl.unwrap_or(false),
                underline = self.val.underline.unwrap_or(false),
                strikethrough = self.val.strikethrough.unwrap_or(false),
                reverse = self.val.reverse.unwrap_or(false),
                nocombine = self.val.nocombine.unwrap_or(false),
            );
            s
        } else {
            let s = format!(
                "vim.api.nvim_set_hl(0,'{name}',{{ fg = '{fg}',bold = {bold},italic = {italic},undercurl = {undercurl},underline = {underline},strikethrough = {strikethrough},reverse = {reverse},nocombine = {nocombine}}})",
                name = self.name,
                fg = self.val.fg,
                bold = self.val.bold.unwrap_or(false),
                italic = self.val.italic.unwrap_or(false),
                undercurl = self.val.undercurl.unwrap_or(false),
                underline = self.val.underline.unwrap_or(false),
                strikethrough = self.val.strikethrough.unwrap_or(false),
                reverse = self.val.reverse.unwrap_or(false),
                nocombine = self.val.nocombine.unwrap_or(false),
            );
            s
        };

        r.replace("bold = false,", "")
            .replace("italic = false,", "")
            .replace("undercurl = false,", "")
            .replace("underline = false,", "")
            .replace("strikethrough = false,", "")
            .replace("reverse = false,", "")
            .replace("nocombine = false", "")
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
struct HighlightDefinition {
    fg: String,
    bg: Option<String>,
    bold: Option<bool>,
    italic: Option<bool>,
    undercurl: Option<bool>,
    underline: Option<bool>,
    strikethrough: Option<bool>,
    reverse: Option<bool>,
    nocombine: Option<bool>,
}

impl HighlightDefinition {
    fn from_map(tbl: &toml::map::Map<std::string::String, Value>, palette: &Palette) -> Self {
        let mut definition = HighlightDefinition::default();
        if !tbl.contains_key("fg") {
            panic!("fg must be set");
        }
        for (gr, v) in tbl.iter() {
            match gr.as_str() {
                "fg" | "foreground" => {
                    if v.is_str() {
                        if v.to_string().replace("\"", "").starts_with('#') {
                            definition.fg = v.to_string().replace("\"", ""); //idk why it formats like "\"#{color}"
                        } else {
                            if let Some(c) = palette.get(&String::from(v.as_str().unwrap())) {
                                definition.fg = c.to_string();
                            } else {
                                panic!("paleete")
                            }
                        }
                    } else {
                        panic!("foreground must be specified by string")
                    }
                }
                "bg" | "background" => {
                    if !v.is_str() {
                        panic!("bg must be specified by string")
                    } else {
                        definition.bg = Some(v.to_string().replace("\"", ""));
                    }
                }
                "bold" => {
                    if !v.is_bool() {
                        panic!("bold must be specified by bool")
                    } else {
                        definition.bold = Some(v.as_bool().unwrap());
                    }
                }
                "italic" => {
                    if !v.is_bool() {
                        panic!("italic must be specified by bool")
                    } else {
                        definition.italic = Some(v.as_bool().unwrap());
                    }
                }
                "undercurl" => {
                    if !v.is_bool() {
                        panic!("undercurl must be specified by bool")
                    } else {
                        definition.undercurl = Some(v.as_bool().unwrap());
                    }
                }
                "underline" => {
                    if !v.is_bool() {
                        panic!("underline must be specified by bool")
                    } else {
                        definition.underline = Some(v.as_bool().unwrap());
                    }
                }
                "strikethrough" => {
                    if !v.is_bool() {
                        panic!("strikethrough must be specified by bool")
                    } else {
                        definition.strikethrough = Some(v.as_bool().unwrap());
                    }
                }
                "reverse" => {
                    if !v.is_bool() {
                        panic!("reverse must be specified by bool")
                    } else {
                        definition.reverse = Some(v.as_bool().unwrap());
                    }
                }
                "nocombine" => {
                    if !v.is_bool() {
                        panic!("nocombine must be specified by bool")
                    } else {
                        definition.nocombine = Some(v.as_bool().unwrap());
                    }
                }

                _ => panic!("cant find this highlight group"),
            };
        }

        definition
    }
}

fn main() {
    let content = std::fs::read_to_string("test.toml").expect("invalid filename");
    let temp = content.parse::<Value>().expect("Invalid Toml");
    let palette = Palette::set(&temp);

    println!("{:?}", palette);
    let highlights = Highlights::set(&temp, &palette);
    for v in highlights.inner.iter() {
        println!("{}", v.write_string_lua());
    }
}

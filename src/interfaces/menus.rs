use dialoguer::{Select, MultiSelect};
use dialoguer::theme::ColorfulTheme;
use clearscreen;
use console::Style;

pub fn menu_generator<'a>(prompt: &str, options: &Vec<&'a str>) -> &'a str {

    let mut theme = ColorfulTheme::default();

    theme.active_item_style = Style::new().black().bold().on_white();
    theme.inactive_item_style = Style::new().white();
    theme.prompt_style = Style::new().white().bold();
    theme.prompt_prefix = Style::new().apply_to("".to_string());
    theme.prompt_suffix = Style::new().apply_to("".to_string());
    theme.active_item_prefix = Style::new().white().bold().apply_to(">".to_string());
    theme.inactive_item_prefix = Style::new().apply_to(" ".to_string());

    let selection = Select::with_theme(&theme)
        .with_prompt(prompt)
        .items(options)
        .default(0)
        .interact()
        .unwrap();

    clearscreen::clear().expect("Failed clearscreen");
    options[selection]
}

pub fn menu_generator_multi<'a>(prompt: &str, options: &Vec<&'a str>) -> Vec<usize> {
    let mut theme = ColorfulTheme::default();

    theme.active_item_style = Style::new().black().bold().on_white();
    theme.inactive_item_style = Style::new().white();
    theme.prompt_style = Style::new().white().bold();
    theme.prompt_prefix = Style::new().apply_to("".to_string());
    theme.prompt_suffix = Style::new().apply_to("".to_string());
    theme.active_item_prefix = Style::new().white().bold().apply_to(">".to_string());
    theme.inactive_item_prefix = Style::new().apply_to(" ".to_string());
    loop {
        
        let selections = MultiSelect::with_theme(&theme)
            .with_prompt(prompt)
            .items(options)
            .interact()
            .unwrap();

        if selections.len() > 2 {
            clearscreen::clear().expect("Failed clearscreen");
            println!("Error: You selected too many items. Please select a maximum of {} items.", 2);
            continue
        } else {
            clearscreen::clear().expect("Failed clearscreen");
            return selections;
        }
    }

}
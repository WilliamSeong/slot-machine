use dialoguer::Select;
use dialoguer::theme::ColorfulTheme;
use clearscreen;

pub fn menu_generator<'a>(prompt: &str, options: &Vec<&'a str>) -> &'a str {
    use console::Style;

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
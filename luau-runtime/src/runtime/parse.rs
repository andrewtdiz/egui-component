use mlua::{MultiValue, Table, Value};

use super::{
    api::{
        function_spec, parse_button_variant as parse_button_variant_token,
        parse_control_size as parse_control_size_token, parse_label_tone as parse_label_tone_token,
        parse_label_weight as parse_label_weight_token,
        parse_number_input_axis as parse_number_input_axis_token,
        parse_select_variant as parse_select_variant_token,
        parse_skeleton_shape as parse_skeleton_shape_token,
        parse_tabs_variant as parse_tabs_variant_token,
        parse_tooltip_placement as parse_tooltip_placement_token, prop_schema_spec,
        RuntimeApiFunctionId, RuntimeApiPropSchemaId, RuntimeApiPropSchemaSpec,
    },
    runtime_lua_error, UiButtonGroupOptions, UiButtonOptions, UiButtonVariant, UiCardOptions,
    UiCheckboxOptions, UiCollapsibleOptions, UiContainerOptions, UiControlSize,
    UiDropdownMenuAction, UiDropdownMenuEntry, UiDropdownMenuOptions, UiDropdownMenuSubmenu,
    UiLabelOptions, UiLabelTone, UiLabelWeight, UiNumberInputAxis, UiNumberInputOptions,
    UiProgressOptions, UiRadioOptions, UiSelectOptions, UiSelectVariant, UiSkeletonOptions,
    UiSkeletonShape, UiSliderOptions, UiSpinnerOptions, UiSwitchOptions, UiTabOption,
    UiTabsOptions, UiTabsVariant, UiTextEditOptions, UiTooltipOptions, UiTooltipPlacement,
    UiVirtualListOptions,
};

fn parse_optional_table_value(
    value: Option<Value>,
    fn_name: &str,
    index: usize,
) -> mlua::Result<Option<Table>> {
    match value {
        None | Some(Value::Nil) => Ok(None),
        Some(Value::Table(table)) => Ok(Some(table)),
        Some(other) => Err(runtime_lua_error(format!(
            "{fn_name} expected argument {index} to be an options table or nil, got {other:?}"
        ))),
    }
}

pub(super) fn parse_label_call(args: MultiValue) -> mlua::Result<(String, Option<Table>)> {
    let function = function_spec(RuntimeApiFunctionId::UiLabel);
    let mut args = args.into_vec().into_iter();
    let text = expect_string_arg(args.next(), function.full_name, 1)?;
    let props = parse_optional_table_value(args.next(), function.full_name, 2)?;
    ensure_no_extra_args(function.full_name, args)?;
    Ok((text, props))
}

fn ensure_known_prop_keys(
    table: &Table,
    fn_name: &str,
    schema: RuntimeApiPropSchemaSpec,
) -> mlua::Result<()> {
    let allowed_list = schema
        .fields
        .iter()
        .map(|field| field.name)
        .collect::<Vec<_>>()
        .join(", ");
    for pair in table.clone().pairs::<Value, Value>() {
        let (key, _) = pair?;
        let key = match key {
            Value::String(value) => value.to_str()?.to_owned(),
            other => {
                return Err(runtime_lua_error(format!(
                    "{fn_name} expected props keys to be strings, got {other:?}"
                )))
            }
        };

        if !schema.fields.iter().any(|field| field.name == key) {
            return Err(runtime_lua_error(format!(
                "{fn_name} received unknown prop `{key}`; allowed keys: {allowed_list}"
            )));
        }
    }
    Ok(())
}

pub(super) fn parse_label_options(table: Option<Table>) -> mlua::Result<UiLabelOptions> {
    let function = function_spec(RuntimeApiFunctionId::UiLabel);
    let Some(table) = table else {
        return Ok(UiLabelOptions::default());
    };

    ensure_known_prop_keys(
        &table,
        function.full_name,
        prop_schema_spec(RuntimeApiPropSchemaId::LabelProps),
    )?;
    let mut options = UiLabelOptions::default();
    if let Some(tone) = raw_string_field(&table, "tone")? {
        options.tone = parse_label_tone(&tone)?;
    }
    if let Some(weight) = raw_string_field(&table, "weight")? {
        options.weight = parse_label_weight(&weight)?;
    }
    options.size = raw_number_field(&table, "size")?;
    Ok(options)
}

pub(super) fn parse_button_call(
    args: MultiValue,
) -> mlua::Result<(String, String, UiButtonOptions)> {
    let function = function_spec(RuntimeApiFunctionId::UiButton);
    let mut args = args.into_vec().into_iter();
    let id = expect_string_arg(args.next(), function.full_name, 1)?;
    let text = expect_string_arg(args.next(), function.full_name, 2)?;
    let options = parse_button_options(match args.next() {
        None | Some(Value::Nil) => None,
        Some(Value::Table(table)) => Some(table),
        Some(other) => {
            return Err(runtime_lua_error(format!(
                "{} expected an options table or nil as the third argument, got {other:?}",
                function.full_name
            )))
        }
    })?;
    ensure_no_extra_args(function.full_name, args)?;
    Ok((id, text, options))
}

fn parse_button_options(table: Option<Table>) -> mlua::Result<UiButtonOptions> {
    let function = function_spec(RuntimeApiFunctionId::UiButton);
    let Some(table) = table else {
        return Ok(UiButtonOptions::default());
    };

    ensure_known_prop_keys(
        &table,
        function.full_name,
        prop_schema_spec(RuntimeApiPropSchemaId::ButtonProps),
    )?;
    let mut options = UiButtonOptions::default();
    if let Some(variant) = raw_string_field(&table, "variant")? {
        options.variant = parse_button_variant(&variant)?;
    }
    if let Some(size) = raw_string_field(&table, "size")? {
        options.size = parse_control_size(&size)?;
    }
    options.width = raw_number_field(&table, "width")?;
    options.leading_icon = raw_string_field(&table, "leading_icon")?;
    options.trailing_icon = raw_string_field(&table, "trailing_icon")?;
    options.icon_size = raw_number_field(&table, "icon_size")?;
    options.icon_only = raw_bool_field(&table, "icon_only")?.unwrap_or(false);
    options.selected = raw_bool_field(&table, "selected")?.unwrap_or(false);
    Ok(options)
}

pub(super) fn parse_text_edit_call(
    args: MultiValue,
) -> mlua::Result<(String, String, UiTextEditOptions)> {
    let function = function_spec(RuntimeApiFunctionId::UiTextEdit);
    let mut args = args.into_vec().into_iter();
    let id = expect_string_arg(args.next(), function.full_name, 1)?;
    let value = expect_string_arg(args.next(), function.full_name, 2)?;
    let options = parse_text_edit_options(match args.next() {
        None | Some(Value::Nil) => None,
        Some(Value::Table(table)) => Some(table),
        Some(other) => {
            return Err(runtime_lua_error(format!(
                "{} expected an options table or nil as the third argument, got {other:?}",
                function.full_name
            )))
        }
    })?;
    ensure_no_extra_args(function.full_name, args)?;
    Ok((id, value, options))
}

pub(super) fn parse_checkbox_call(
    args: MultiValue,
) -> mlua::Result<(String, bool, UiCheckboxOptions)> {
    let function = function_spec(RuntimeApiFunctionId::UiCheckbox);
    let mut args = args.into_vec().into_iter();
    let id = expect_string_arg(args.next(), function.full_name, 1)?;
    let checked = expect_bool_arg(args.next(), function.full_name, 2)?;
    let options = parse_checkbox_options(match args.next() {
        None | Some(Value::Nil) => None,
        Some(Value::Table(table)) => Some(table),
        Some(other) => {
            return Err(runtime_lua_error(format!(
                "{} expected an options table or nil as the third argument, got {other:?}",
                function.full_name
            )))
        }
    })?;
    ensure_no_extra_args(function.full_name, args)?;
    Ok((id, checked, options))
}

pub(super) fn parse_switch_call(args: MultiValue) -> mlua::Result<(String, bool, UiSwitchOptions)> {
    let function = function_spec(RuntimeApiFunctionId::UiSwitch);
    let mut args = args.into_vec().into_iter();
    let id = expect_string_arg(args.next(), function.full_name, 1)?;
    let checked = expect_bool_arg(args.next(), function.full_name, 2)?;
    let options = parse_switch_options(match args.next() {
        None | Some(Value::Nil) => None,
        Some(Value::Table(table)) => Some(table),
        Some(other) => {
            return Err(runtime_lua_error(format!(
                "{} expected an options table or nil as the third argument, got {other:?}",
                function.full_name
            )))
        }
    })?;
    ensure_no_extra_args(function.full_name, args)?;
    Ok((id, checked, options))
}

pub(super) fn parse_slider_call(args: MultiValue) -> mlua::Result<(String, f32, UiSliderOptions)> {
    let function = function_spec(RuntimeApiFunctionId::UiSlider);
    let mut args = args.into_vec().into_iter();
    let id = expect_string_arg(args.next(), function.full_name, 1)?;
    let value = expect_number_arg(args.next(), function.full_name, 2)?;
    let options = match args.next() {
        Some(Value::Table(table)) => parse_slider_options(table)?,
        Some(other) => {
            return Err(runtime_lua_error(format!(
                "{} expected an options table as the third argument, got {other:?}",
                function.full_name
            )))
        }
        None => {
            return Err(runtime_lua_error(format!(
                "{} is missing required argument 3",
                function.full_name
            )))
        }
    };
    ensure_no_extra_args(function.full_name, args)?;
    Ok((id, value, options))
}

pub(super) fn parse_number_input_call(
    args: MultiValue,
) -> mlua::Result<(String, f32, UiNumberInputOptions)> {
    let function = function_spec(RuntimeApiFunctionId::UiNumberInput);
    let mut args = args.into_vec().into_iter();
    let id = expect_string_arg(args.next(), function.full_name, 1)?;
    let value = expect_number_arg(args.next(), function.full_name, 2)?;
    let options = parse_number_input_options(match args.next() {
        None | Some(Value::Nil) => None,
        Some(Value::Table(table)) => Some(table),
        Some(other) => {
            return Err(runtime_lua_error(format!(
                "{} expected an options table or nil as the third argument, got {other:?}",
                function.full_name
            )))
        }
    })?;
    ensure_no_extra_args(function.full_name, args)?;
    Ok((id, value, options))
}

pub(super) fn parse_select_call(
    args: MultiValue,
) -> mlua::Result<(String, usize, Vec<String>, UiSelectOptions)> {
    let function = function_spec(RuntimeApiFunctionId::UiSelect);
    let mut args = args.into_vec().into_iter();
    let id = expect_string_arg(args.next(), function.full_name, 1)?;
    let selected_index = expect_index_arg(args.next(), function.full_name, 2)?;
    let options = expect_string_object_list_arg(args.next(), function.full_name, 3, "label")?;
    let props = parse_select_options(match args.next() {
        None | Some(Value::Nil) => None,
        Some(Value::Table(table)) => Some(table),
        Some(other) => {
            return Err(runtime_lua_error(format!(
                "{} expected an options table or nil as the fourth argument, got {other:?}",
                function.full_name
            )))
        }
    })?;
    ensure_no_extra_args(function.full_name, args)?;
    Ok((id, selected_index, options, props))
}

pub(super) fn parse_tabs_call(
    args: MultiValue,
) -> mlua::Result<(String, usize, Vec<UiTabOption>, UiTabsOptions)> {
    let function = function_spec(RuntimeApiFunctionId::UiTabs);
    let mut args = args.into_vec().into_iter();
    let id = expect_string_arg(args.next(), function.full_name, 1)?;
    let selected_index = expect_index_arg(args.next(), function.full_name, 2)?;
    let options = expect_tab_options_arg(args.next(), function.full_name, 3)?;
    let props = parse_tabs_options(match args.next() {
        None | Some(Value::Nil) => None,
        Some(Value::Table(table)) => Some(table),
        Some(other) => {
            return Err(runtime_lua_error(format!(
                "{} expected an options table or nil as the fourth argument, got {other:?}",
                function.full_name
            )))
        }
    })?;
    ensure_no_extra_args(function.full_name, args)?;
    Ok((id, selected_index, options, props))
}

pub(super) fn parse_progress_call(args: MultiValue) -> mlua::Result<(f32, UiProgressOptions)> {
    let function = function_spec(RuntimeApiFunctionId::UiProgress);
    let mut args = args.into_vec().into_iter();
    let value = expect_number_arg(args.next(), function.full_name, 1)?;
    let props = parse_progress_options(match args.next() {
        None | Some(Value::Nil) => None,
        Some(Value::Table(table)) => Some(table),
        Some(other) => {
            return Err(runtime_lua_error(format!(
                "{} expected an options table or nil as the second argument, got {other:?}",
                function.full_name
            )))
        }
    })?;
    ensure_no_extra_args(function.full_name, args)?;
    Ok((value, props))
}

pub(super) fn parse_radio_call(args: MultiValue) -> mlua::Result<(String, bool, UiRadioOptions)> {
    let function = function_spec(RuntimeApiFunctionId::UiRadio);
    let mut args = args.into_vec().into_iter();
    let id = expect_string_arg(args.next(), function.full_name, 1)?;
    let selected = expect_bool_arg(args.next(), function.full_name, 2)?;
    let props = parse_radio_options(match args.next() {
        None | Some(Value::Nil) => None,
        Some(Value::Table(table)) => Some(table),
        Some(other) => {
            return Err(runtime_lua_error(format!(
                "{} expected an options table or nil as the third argument, got {other:?}",
                function.full_name
            )))
        }
    })?;
    ensure_no_extra_args(function.full_name, args)?;
    Ok((id, selected, props))
}

pub(super) fn parse_button_group_call(
    args: MultiValue,
) -> mlua::Result<(String, Vec<String>, UiButtonGroupOptions)> {
    let function = function_spec(RuntimeApiFunctionId::UiButtonGroup);
    let mut args = args.into_vec().into_iter();
    let id = expect_string_arg(args.next(), function.full_name, 1)?;
    let options = expect_string_object_list_arg(args.next(), function.full_name, 2, "label")?;
    let props = parse_button_group_options(match args.next() {
        None | Some(Value::Nil) => None,
        Some(Value::Table(table)) => Some(table),
        Some(other) => {
            return Err(runtime_lua_error(format!(
                "{} expected an options table or nil as the third argument, got {other:?}",
                function.full_name
            )))
        }
    })?;
    ensure_no_extra_args(function.full_name, args)?;
    Ok((id, options, props))
}

pub(super) fn parse_begin_collapsible_call(
    args: MultiValue,
) -> mlua::Result<(String, String, bool, UiCollapsibleOptions)> {
    let function = function_spec(RuntimeApiFunctionId::UiBeginCollapsible);
    let mut args = args.into_vec().into_iter();
    let id = expect_string_arg(args.next(), function.full_name, 1)?;
    let title = expect_string_arg(args.next(), function.full_name, 2)?;
    let open = expect_bool_arg(args.next(), function.full_name, 3)?;
    let props = parse_collapsible_options(match args.next() {
        None | Some(Value::Nil) => None,
        Some(Value::Table(table)) => Some(table),
        Some(other) => {
            return Err(runtime_lua_error(format!(
                "{} expected an options table or nil as the fourth argument, got {other:?}",
                function.full_name
            )))
        }
    })?;
    ensure_no_extra_args(function.full_name, args)?;
    Ok((id, title, open, props))
}

pub(super) fn parse_dropdown_menu_call(
    args: MultiValue,
) -> mlua::Result<(
    String,
    String,
    Vec<UiDropdownMenuEntry>,
    UiDropdownMenuOptions,
)> {
    let function = function_spec(RuntimeApiFunctionId::UiDropdownMenu);
    let mut args = args.into_vec().into_iter();
    let id = expect_string_arg(args.next(), function.full_name, 1)?;
    let trigger_label = expect_string_arg(args.next(), function.full_name, 2)?;
    let entries = expect_dropdown_menu_entries_arg(args.next(), function.full_name, 3)?;
    let props = parse_dropdown_menu_options(match args.next() {
        None | Some(Value::Nil) => None,
        Some(Value::Table(table)) => Some(table),
        Some(other) => {
            return Err(runtime_lua_error(format!(
                "{} expected an options table or nil as the fourth argument, got {other:?}",
                function.full_name
            )))
        }
    })?;
    ensure_no_extra_args(function.full_name, args)?;
    Ok((id, trigger_label, entries, props))
}

pub(super) fn parse_tooltip_call(
    args: MultiValue,
) -> mlua::Result<(String, String, UiTooltipOptions)> {
    let function = function_spec(RuntimeApiFunctionId::UiTooltip);
    let mut args = args.into_vec().into_iter();
    let trigger_label = expect_string_arg(args.next(), function.full_name, 1)?;
    let text = expect_string_arg(args.next(), function.full_name, 2)?;
    let props = parse_tooltip_options(match args.next() {
        None | Some(Value::Nil) => None,
        Some(Value::Table(table)) => Some(table),
        Some(other) => {
            return Err(runtime_lua_error(format!(
                "{} expected an options table or nil as the third argument, got {other:?}",
                function.full_name
            )))
        }
    })?;
    ensure_no_extra_args(function.full_name, args)?;
    Ok((trigger_label, text, props))
}

pub(super) fn parse_spinner_call(args: MultiValue) -> mlua::Result<UiSpinnerOptions> {
    let function = function_spec(RuntimeApiFunctionId::UiSpinner);
    parse_spinner_options(parse_optional_table_arg(args, function.full_name)?)
}

pub(super) fn parse_skeleton_call(args: MultiValue) -> mlua::Result<UiSkeletonOptions> {
    let function = function_spec(RuntimeApiFunctionId::UiSkeleton);
    parse_skeleton_options(parse_optional_table_arg(args, function.full_name)?)
}

pub(super) fn parse_virtual_list_call(
    args: MultiValue,
) -> mlua::Result<(String, Table, UiVirtualListOptions)> {
    let function = function_spec(RuntimeApiFunctionId::UiVirtualList);
    let mut args = args.into_vec().into_iter();
    let id = expect_string_arg(args.next(), function.full_name, 1)?;
    let items = expect_string_list_table_arg(args.next(), function.full_name, 2)?;
    let options = parse_virtual_list_options(match args.next() {
        None | Some(Value::Nil) => None,
        Some(Value::Table(table)) => Some(table),
        Some(other) => {
            return Err(runtime_lua_error(format!(
                "{} expected an options table or nil as the third argument, got {other:?}",
                function.full_name
            )))
        }
    })?;
    ensure_no_extra_args(function.full_name, args)?;
    Ok((id, items, options))
}

fn parse_text_edit_options(table: Option<Table>) -> mlua::Result<UiTextEditOptions> {
    let function = function_spec(RuntimeApiFunctionId::UiTextEdit);
    let Some(table) = table else {
        return Ok(UiTextEditOptions::default());
    };

    ensure_known_prop_keys(
        &table,
        function.full_name,
        prop_schema_spec(RuntimeApiPropSchemaId::TextEditProps),
    )?;
    Ok(UiTextEditOptions {
        width: raw_number_field(&table, "width")?,
        placeholder: raw_string_field(&table, "placeholder")?,
        leading_icon: raw_string_field(&table, "leading_icon")?,
        password: raw_bool_field(&table, "password")?.unwrap_or(false),
    })
}

fn parse_checkbox_options(table: Option<Table>) -> mlua::Result<UiCheckboxOptions> {
    let function = function_spec(RuntimeApiFunctionId::UiCheckbox);
    let Some(table) = table else {
        return Ok(UiCheckboxOptions::default());
    };
    ensure_known_prop_keys(
        &table,
        function.full_name,
        prop_schema_spec(RuntimeApiPropSchemaId::CheckboxProps),
    )?;
    Ok(UiCheckboxOptions {
        label: raw_string_field(&table, "label")?,
    })
}

fn parse_switch_options(table: Option<Table>) -> mlua::Result<UiSwitchOptions> {
    let function = function_spec(RuntimeApiFunctionId::UiSwitch);
    let Some(table) = table else {
        return Ok(UiSwitchOptions::default());
    };
    ensure_known_prop_keys(
        &table,
        function.full_name,
        prop_schema_spec(RuntimeApiPropSchemaId::SwitchProps),
    )?;
    let mut options = UiSwitchOptions::default();
    options.label = raw_string_field(&table, "label")?;
    if let Some(size) = raw_string_field(&table, "size")? {
        options.size = parse_control_size(&size)?;
    }
    Ok(options)
}

fn parse_slider_options(table: Table) -> mlua::Result<UiSliderOptions> {
    let function = function_spec(RuntimeApiFunctionId::UiSlider);
    ensure_known_prop_keys(
        &table,
        function.full_name,
        prop_schema_spec(RuntimeApiPropSchemaId::SliderProps),
    )?;
    let min = raw_number_field(&table, "min")?
        .ok_or_else(|| runtime_lua_error(format!("{} props require `min`", function.full_name)))?;
    let max = raw_number_field(&table, "max")?
        .ok_or_else(|| runtime_lua_error(format!("{} props require `max`", function.full_name)))?;
    Ok(UiSliderOptions {
        min,
        max,
        width: raw_number_field(&table, "width")?,
    })
}

fn parse_number_input_options(table: Option<Table>) -> mlua::Result<UiNumberInputOptions> {
    let function = function_spec(RuntimeApiFunctionId::UiNumberInput);
    let Some(table) = table else {
        return Ok(UiNumberInputOptions::default());
    };
    ensure_known_prop_keys(
        &table,
        function.full_name,
        prop_schema_spec(RuntimeApiPropSchemaId::NumberInputProps),
    )?;
    let mut options = UiNumberInputOptions::default();
    options.min = raw_number_field(&table, "min")?;
    options.max = raw_number_field(&table, "max")?;
    options.width = raw_number_field(&table, "width")?;
    options.speed = raw_f64_field(&table, "speed")?;
    options.fine_speed = raw_f64_field(&table, "fine_speed")?;
    options.decimals = raw_usize_field(&table, "decimals")?;
    options.fine_decimals = raw_usize_field(&table, "fine_decimals")?;
    options.prefix = raw_string_field(&table, "prefix")?;
    options.suffix = raw_string_field(&table, "suffix")?;
    options.prefix_tint = raw_string_field(&table, "prefix_tint")?;
    options.prefix_align_left = raw_bool_field(&table, "prefix_align_left")?.unwrap_or(false);
    if let Some(axis) = raw_string_field(&table, "axis")? {
        options.axis = parse_number_input_axis(&axis)?;
    }
    Ok(options)
}

fn parse_select_options(table: Option<Table>) -> mlua::Result<UiSelectOptions> {
    let function = function_spec(RuntimeApiFunctionId::UiSelect);
    let Some(table) = table else {
        return Ok(UiSelectOptions::default());
    };
    ensure_known_prop_keys(
        &table,
        function.full_name,
        prop_schema_spec(RuntimeApiPropSchemaId::SelectProps),
    )?;
    let mut options = UiSelectOptions::default();
    options.width = raw_number_field(&table, "width")?;
    options.placeholder = raw_string_field(&table, "placeholder")?;
    if let Some(variant) = raw_string_field(&table, "variant")? {
        options.variant = parse_select_variant(&variant)?;
    }
    Ok(options)
}

fn parse_tabs_options(table: Option<Table>) -> mlua::Result<UiTabsOptions> {
    let function = function_spec(RuntimeApiFunctionId::UiTabs);
    let Some(table) = table else {
        return Ok(UiTabsOptions::default());
    };
    ensure_known_prop_keys(
        &table,
        function.full_name,
        prop_schema_spec(RuntimeApiPropSchemaId::TabsProps),
    )?;
    let mut options = UiTabsOptions::default();
    if let Some(variant) = raw_string_field(&table, "variant")? {
        options.variant = parse_tabs_variant(&variant)?;
    }
    Ok(options)
}

fn parse_progress_options(table: Option<Table>) -> mlua::Result<UiProgressOptions> {
    let function = function_spec(RuntimeApiFunctionId::UiProgress);
    let Some(table) = table else {
        return Ok(UiProgressOptions::default());
    };
    ensure_known_prop_keys(
        &table,
        function.full_name,
        prop_schema_spec(RuntimeApiPropSchemaId::ProgressProps),
    )?;
    Ok(UiProgressOptions {
        width: raw_number_field(&table, "width")?,
        height: raw_number_field(&table, "height")?,
    })
}

fn parse_radio_options(table: Option<Table>) -> mlua::Result<UiRadioOptions> {
    let function = function_spec(RuntimeApiFunctionId::UiRadio);
    let Some(table) = table else {
        return Ok(UiRadioOptions::default());
    };
    ensure_known_prop_keys(
        &table,
        function.full_name,
        prop_schema_spec(RuntimeApiPropSchemaId::RadioProps),
    )?;
    Ok(UiRadioOptions {
        label: raw_string_field(&table, "label")?,
        description: raw_string_field(&table, "description")?,
    })
}

fn parse_button_group_options(table: Option<Table>) -> mlua::Result<UiButtonGroupOptions> {
    let function = function_spec(RuntimeApiFunctionId::UiButtonGroup);
    let Some(table) = table else {
        return Ok(UiButtonGroupOptions::default());
    };
    ensure_known_prop_keys(
        &table,
        function.full_name,
        prop_schema_spec(RuntimeApiPropSchemaId::ButtonGroupProps),
    )?;
    Ok(UiButtonGroupOptions::default())
}

fn parse_collapsible_options(table: Option<Table>) -> mlua::Result<UiCollapsibleOptions> {
    let function = function_spec(RuntimeApiFunctionId::UiBeginCollapsible);
    let Some(table) = table else {
        return Ok(UiCollapsibleOptions::default());
    };
    ensure_known_prop_keys(
        &table,
        function.full_name,
        prop_schema_spec(RuntimeApiPropSchemaId::CollapsibleProps),
    )?;
    Ok(UiCollapsibleOptions {
        open: raw_bool_field(&table, "open")?,
        leading_icon: raw_string_field(&table, "leading_icon")?,
        trailing_icon: raw_string_field(&table, "trailing_icon")?,
    })
}

fn parse_dropdown_menu_options(table: Option<Table>) -> mlua::Result<UiDropdownMenuOptions> {
    let function = function_spec(RuntimeApiFunctionId::UiDropdownMenu);
    let Some(table) = table else {
        return Ok(UiDropdownMenuOptions::default());
    };
    ensure_known_prop_keys(
        &table,
        function.full_name,
        prop_schema_spec(RuntimeApiPropSchemaId::DropdownMenuProps),
    )?;
    let mut options = UiDropdownMenuOptions::default();
    options.width = raw_number_field(&table, "width")?;
    if let Some(variant) = raw_string_field(&table, "trigger_variant")? {
        options.trigger_variant = parse_button_variant(&variant)?;
    }
    Ok(options)
}

fn parse_tooltip_options(table: Option<Table>) -> mlua::Result<UiTooltipOptions> {
    let function = function_spec(RuntimeApiFunctionId::UiTooltip);
    let Some(table) = table else {
        return Ok(UiTooltipOptions::default());
    };
    ensure_known_prop_keys(
        &table,
        function.full_name,
        prop_schema_spec(RuntimeApiPropSchemaId::TooltipProps),
    )?;
    let mut options = UiTooltipOptions::default();
    options.width = raw_number_field(&table, "width")?;
    options.delay_ms = raw_u32_field(&table, "delay_ms")?;
    if let Some(placement) = raw_string_field(&table, "placement")? {
        options.placement = parse_tooltip_placement(&placement)?;
    }
    Ok(options)
}

fn parse_spinner_options(table: Option<Table>) -> mlua::Result<UiSpinnerOptions> {
    let function = function_spec(RuntimeApiFunctionId::UiSpinner);
    let Some(table) = table else {
        return Ok(UiSpinnerOptions::default());
    };
    ensure_known_prop_keys(
        &table,
        function.full_name,
        prop_schema_spec(RuntimeApiPropSchemaId::SpinnerProps),
    )?;
    Ok(UiSpinnerOptions {
        size: raw_number_field(&table, "size")?,
        stroke_width: raw_number_field(&table, "stroke_width")?,
        speed: raw_number_field(&table, "speed")?,
        color: raw_string_field(&table, "color")?,
    })
}

fn parse_skeleton_options(table: Option<Table>) -> mlua::Result<UiSkeletonOptions> {
    let function = function_spec(RuntimeApiFunctionId::UiSkeleton);
    let Some(table) = table else {
        return Ok(UiSkeletonOptions::default());
    };
    ensure_known_prop_keys(
        &table,
        function.full_name,
        prop_schema_spec(RuntimeApiPropSchemaId::SkeletonProps),
    )?;
    let mut options = UiSkeletonOptions::default();
    options.width = raw_number_field(&table, "width")?;
    options.height = raw_number_field(&table, "height")?;
    if let Some(shape) = raw_string_field(&table, "shape")? {
        options.shape = parse_skeleton_shape(&shape)?;
    }
    Ok(options)
}

fn parse_virtual_list_options(table: Option<Table>) -> mlua::Result<UiVirtualListOptions> {
    let function = function_spec(RuntimeApiFunctionId::UiVirtualList);
    let Some(table) = table else {
        return Ok(UiVirtualListOptions::default());
    };

    ensure_known_prop_keys(
        &table,
        function.full_name,
        prop_schema_spec(RuntimeApiPropSchemaId::VirtualListProps),
    )?;
    let mut options = UiVirtualListOptions::default();
    options.width = raw_number_field(&table, "width")?;
    options.height = raw_number_field(&table, "height")?;
    if let Some(row_height) = raw_number_field(&table, "row_height")? {
        options.row_height = row_height;
    }
    if let Some(selected_index) = raw_index_field(&table, "selected_index")? {
        options.selected_index = selected_index;
    }
    Ok(options)
}

pub(super) fn parse_optional_table_arg(
    args: MultiValue,
    fn_name: &str,
) -> mlua::Result<Option<Table>> {
    let mut args = args.into_vec().into_iter();
    let table = match args.next() {
        None | Some(Value::Nil) => None,
        Some(Value::Table(table)) => Some(table),
        Some(other) => {
            return Err(runtime_lua_error(format!(
                "{fn_name} expected an options table or nil as the first argument, got {other:?}"
            )))
        }
    };
    ensure_no_extra_args(fn_name, args)?;
    Ok(table)
}

pub(super) fn parse_container_options(
    table: Option<Table>,
    fn_name: &str,
) -> mlua::Result<UiContainerOptions> {
    let Some(table) = table else {
        return Ok(UiContainerOptions::default());
    };

    ensure_known_prop_keys(
        &table,
        fn_name,
        prop_schema_spec(RuntimeApiPropSchemaId::ContainerProps),
    )?;
    Ok(UiContainerOptions {
        gap: raw_number_field(&table, "gap")?,
    })
}

pub(super) fn parse_card_options(
    table: Option<Table>,
    fn_name: &str,
) -> mlua::Result<UiCardOptions> {
    let Some(table) = table else {
        return Ok(UiCardOptions::default());
    };

    ensure_known_prop_keys(
        &table,
        fn_name,
        prop_schema_spec(RuntimeApiPropSchemaId::CardProps),
    )?;
    let mut options = UiCardOptions::default();
    options.width = raw_number_field(&table, "width")?;
    if let Some(padding_x) = raw_number_field(&table, "padding_x")? {
        options.padding_x = padding_x;
    }
    if let Some(padding_y) = raw_number_field(&table, "padding_y")? {
        options.padding_y = padding_y;
    }
    Ok(options)
}

pub(super) fn ensure_no_extra_args(
    fn_name: &str,
    mut extras: impl Iterator<Item = Value>,
) -> mlua::Result<()> {
    if let Some(extra) = extras.next() {
        return Err(runtime_lua_error(format!(
            "{fn_name} received an unexpected extra argument: {extra:?}"
        )));
    }
    Ok(())
}

pub(super) fn expect_string_arg(
    value: Option<Value>,
    fn_name: &str,
    index: usize,
) -> mlua::Result<String> {
    match value {
        Some(Value::String(value)) => Ok(value.to_str()?.to_owned()),
        Some(other) => Err(runtime_lua_error(format!(
            "{fn_name} expected argument {index} to be a string, got {other:?}"
        ))),
        None => Err(runtime_lua_error(format!(
            "{fn_name} is missing required argument {index}"
        ))),
    }
}

fn expect_bool_arg(value: Option<Value>, fn_name: &str, index: usize) -> mlua::Result<bool> {
    match value {
        Some(Value::Boolean(value)) => Ok(value),
        Some(other) => Err(runtime_lua_error(format!(
            "{fn_name} expected argument {index} to be a boolean, got {other:?}"
        ))),
        None => Err(runtime_lua_error(format!(
            "{fn_name} is missing required argument {index}"
        ))),
    }
}

fn expect_number_arg(value: Option<Value>, fn_name: &str, index: usize) -> mlua::Result<f32> {
    match value {
        Some(Value::Integer(value)) => Ok(value as f32),
        Some(Value::Number(value)) => Ok(value as f32),
        Some(other) => Err(runtime_lua_error(format!(
            "{fn_name} expected argument {index} to be a number, got {other:?}"
        ))),
        None => Err(runtime_lua_error(format!(
            "{fn_name} is missing required argument {index}"
        ))),
    }
}

fn expect_index_arg(value: Option<Value>, fn_name: &str, index: usize) -> mlua::Result<usize> {
    match value {
        Some(Value::Integer(value)) if value >= 0 => Ok(value as usize),
        Some(Value::Number(value)) if value >= 0.0 && value.fract() == 0.0 => Ok(value as usize),
        Some(other) => Err(runtime_lua_error(format!(
            "{fn_name} expected argument {index} to be a non-negative integer, got {other:?}"
        ))),
        None => Err(runtime_lua_error(format!(
            "{fn_name} is missing required argument {index}"
        ))),
    }
}

pub(super) fn expect_string_list_table_arg(
    value: Option<Value>,
    fn_name: &str,
    index: usize,
) -> mlua::Result<Table> {
    match value {
        Some(Value::Table(table)) => Ok(table),
        Some(other) => Err(runtime_lua_error(format!(
            "{fn_name} expected argument {index} to be an array of strings, got {other:?}"
        ))),
        None => Err(runtime_lua_error(format!(
            "{fn_name} is missing required argument {index}"
        ))),
    }
}

pub(super) struct ParsedStringListSnapshot {
    pub items: Vec<String>,
    pub item_keys: Vec<usize>,
    pub total_string_bytes: usize,
}

pub(super) fn snapshot_string_list_table(
    table: &Table,
    fn_name: &str,
    index: usize,
) -> mlua::Result<ParsedStringListSnapshot> {
    let len = table.raw_len();
    let mut items = Vec::with_capacity(len);
    let mut item_keys = Vec::with_capacity(len);
    let mut total_string_bytes = 0;
    for i in 1..=len {
        match table.raw_get::<Value>(i)? {
            Value::String(value) => {
                let item = value.to_str()?.to_owned();
                total_string_bytes += item.len();
                item_keys.push(value.to_pointer() as usize);
                items.push(item);
            }
            Value::Nil => {
                return Err(runtime_lua_error(format!(
                    "{fn_name} expected argument {index} to contain a dense string array; index {i} is missing"
                )))
            }
            other => {
                return Err(runtime_lua_error(format!(
                    "{fn_name} expected argument {index} to contain only strings, got {other:?} at index {i}"
                )))
            }
        }
    }

    ensure_dense_integer_keys(table, fn_name, index, len)?;
    Ok(ParsedStringListSnapshot {
        items,
        item_keys,
        total_string_bytes,
    })
}

pub(super) fn string_list_table_changed(
    table: &Table,
    fn_name: &str,
    index: usize,
    expected_keys: &[usize],
) -> mlua::Result<bool> {
    let len = table.raw_len();
    let mut changed = len != expected_keys.len();
    for i in 1..=len {
        match table.raw_get::<Value>(i)? {
            Value::String(value) => {
                let item_key = value.to_pointer() as usize;
                if expected_keys.get(i - 1).copied() != Some(item_key) {
                    changed = true;
                }
            }
            Value::Nil => {
                return Err(runtime_lua_error(format!(
                    "{fn_name} expected argument {index} to contain a dense string array; index {i} is missing"
                )))
            }
            other => {
                return Err(runtime_lua_error(format!(
                    "{fn_name} expected argument {index} to contain only strings, got {other:?} at index {i}"
                )))
            }
        }
    }

    ensure_dense_integer_keys(table, fn_name, index, len)?;
    Ok(changed)
}

fn ensure_dense_integer_keys(
    table: &Table,
    fn_name: &str,
    index: usize,
    len: usize,
) -> mlua::Result<()> {
    for pair in table.clone().pairs::<Value, Value>() {
        let (key, _) = pair?;
        let key_index = match key {
            Value::Integer(value) if value >= 1 => value as usize,
            Value::Number(value) if value >= 1.0 && value.fract() == 0.0 => value as usize,
            other => {
                return Err(runtime_lua_error(format!(
                    "{fn_name} expected argument {index} to use only consecutive integer keys, got {other:?}"
                )))
            }
        };
        if key_index == 0 || key_index > len {
            return Err(runtime_lua_error(format!(
                "{fn_name} expected argument {index} to use only consecutive integer keys 1..{len}, got {key_index}"
            )));
        }
    }
    Ok(())
}

fn expect_object_array_arg(
    value: Option<Value>,
    fn_name: &str,
    index: usize,
) -> mlua::Result<Vec<Table>> {
    let table = match value {
        Some(Value::Table(table)) => table,
        Some(other) => {
            return Err(runtime_lua_error(format!(
                "{fn_name} expected argument {index} to be an array of tables, got {other:?}"
            )))
        }
        None => {
            return Err(runtime_lua_error(format!(
                "{fn_name} is missing required argument {index}"
            )))
        }
    };

    let len = table.raw_len();
    let mut values = Vec::with_capacity(len);
    for i in 1..=len {
        match table.raw_get::<Value>(i)? {
            Value::Table(value) => values.push(value),
            Value::Nil => {
                return Err(runtime_lua_error(format!(
                    "{fn_name} expected argument {index} to contain a dense table array; index {i} is missing"
                )))
            }
            other => {
                return Err(runtime_lua_error(format!(
                    "{fn_name} expected argument {index} to contain only tables, got {other:?} at index {i}"
                )))
            }
        }
    }

    for pair in table.clone().pairs::<Value, Value>() {
        let (key, _) = pair?;
        let key_index = match key {
            Value::Integer(value) if value >= 1 => value as usize,
            Value::Number(value) if value >= 1.0 && value.fract() == 0.0 => value as usize,
            other => {
                return Err(runtime_lua_error(format!(
                    "{fn_name} expected argument {index} to use only consecutive integer keys, got {other:?}"
                )))
            }
        };
        if key_index == 0 || key_index > len {
            return Err(runtime_lua_error(format!(
                "{fn_name} expected argument {index} to use only consecutive integer keys 1..{len}, got {key_index}"
            )));
        }
    }

    Ok(values)
}

fn expect_string_object_list_arg(
    value: Option<Value>,
    fn_name: &str,
    index: usize,
    field_name: &str,
) -> mlua::Result<Vec<String>> {
    expect_object_array_arg(value, fn_name, index)?
        .into_iter()
        .map(|table| {
            ensure_only_fields(&table, fn_name, &[field_name])?;
            raw_string_field(&table, field_name)?.ok_or_else(|| {
                runtime_lua_error(format!(
                    "{fn_name} expected `{field_name}` to be present in each option table"
                ))
            })
        })
        .collect()
}

fn expect_tab_options_arg(
    value: Option<Value>,
    fn_name: &str,
    index: usize,
) -> mlua::Result<Vec<UiTabOption>> {
    expect_object_array_arg(value, fn_name, index)?
        .into_iter()
        .map(|table| {
            ensure_only_fields(&table, fn_name, &["label", "icon", "icon_only", "tooltip"])?;
            let label = raw_string_field(&table, "label")?.ok_or_else(|| {
                runtime_lua_error(format!("{fn_name} expected `label` in each tab option"))
            })?;
            Ok(UiTabOption {
                label,
                icon: raw_string_field(&table, "icon")?,
                icon_only: raw_bool_field(&table, "icon_only")?.unwrap_or(false),
                tooltip: raw_string_field(&table, "tooltip")?,
            })
        })
        .collect()
}

fn expect_dropdown_menu_entries_arg(
    value: Option<Value>,
    fn_name: &str,
    index: usize,
) -> mlua::Result<Vec<UiDropdownMenuEntry>> {
    expect_object_array_arg(value, fn_name, index)?
        .into_iter()
        .map(|table| parse_dropdown_menu_entry(&table, fn_name))
        .collect()
}

fn parse_dropdown_menu_entry(table: &Table, fn_name: &str) -> mlua::Result<UiDropdownMenuEntry> {
    ensure_only_fields(
        table,
        fn_name,
        &[
            "kind", "id", "label", "icon", "shortcut", "enabled", "selected", "entries",
        ],
    )?;
    let kind = raw_string_field(table, "kind")?.ok_or_else(|| {
        runtime_lua_error(format!(
            "{fn_name} expected each dropdown entry to include `kind`"
        ))
    })?;
    match kind.as_str() {
        "action" => Ok(UiDropdownMenuEntry::Action(UiDropdownMenuAction {
            id: raw_usize_field(table, "id")?.ok_or_else(|| {
                runtime_lua_error(format!(
                    "{fn_name} expected action entries to include integer `id`"
                ))
            })?,
            label: raw_string_field(table, "label")?.ok_or_else(|| {
                runtime_lua_error(format!(
                    "{fn_name} expected action entries to include `label`"
                ))
            })?,
            icon: raw_string_field(table, "icon")?,
            shortcut: raw_string_field(table, "shortcut")?,
            enabled: raw_bool_field(table, "enabled")?.unwrap_or(true),
            selected: raw_bool_field(table, "selected")?.unwrap_or(false),
        })),
        "separator" => Ok(UiDropdownMenuEntry::Separator),
        "submenu" => {
            let label = raw_string_field(table, "label")?.ok_or_else(|| {
                runtime_lua_error(format!(
                    "{fn_name} expected submenu entries to include `label`"
                ))
            })?;
            let entries = match table.raw_get::<Value>("entries")? {
                Value::Table(entries) => {
                    expect_dropdown_menu_entries_arg(Some(Value::Table(entries)), fn_name, 0)?
                }
                Value::Nil => {
                    return Err(runtime_lua_error(format!(
                        "{fn_name} expected submenu entries to include `entries`"
                    )))
                }
                other => {
                    return Err(runtime_lua_error(format!(
                        "{fn_name} expected submenu `entries` to be an array of entry tables, got {other:?}"
                    )))
                }
            };
            Ok(UiDropdownMenuEntry::Submenu(UiDropdownMenuSubmenu {
                label,
                icon: raw_string_field(table, "icon")?,
                entries,
            }))
        }
        other => Err(runtime_lua_error(format!(
            "{fn_name} received unknown dropdown entry kind `{other}`"
        ))),
    }
}

fn ensure_only_fields(table: &Table, fn_name: &str, allowed: &[&str]) -> mlua::Result<()> {
    for pair in table.clone().pairs::<Value, Value>() {
        let (key, _) = pair?;
        let key = match key {
            Value::String(value) => value.to_str()?.to_owned(),
            other => {
                return Err(runtime_lua_error(format!(
                    "{fn_name} expected object keys to be strings, got {other:?}"
                )))
            }
        };
        if !allowed.iter().any(|field| *field == key) {
            return Err(runtime_lua_error(format!(
                "{fn_name} received unknown object field `{key}`"
            )));
        }
    }
    Ok(())
}

fn raw_string_field(table: &Table, key: &str) -> mlua::Result<Option<String>> {
    match table.raw_get::<Value>(key)? {
        Value::Nil => Ok(None),
        Value::String(value) => Ok(Some(value.to_str()?.to_owned())),
        other => Err(runtime_lua_error(format!(
            "field `{key}` must be a string when present, got {other:?}"
        ))),
    }
}

fn raw_number_field(table: &Table, key: &str) -> mlua::Result<Option<f32>> {
    match table.raw_get::<Value>(key)? {
        Value::Nil => Ok(None),
        Value::Integer(value) => Ok(Some(value as f32)),
        Value::Number(value) => Ok(Some(value as f32)),
        other => Err(runtime_lua_error(format!(
            "field `{key}` must be a number when present, got {other:?}"
        ))),
    }
}

fn raw_f64_field(table: &Table, key: &str) -> mlua::Result<Option<f64>> {
    match table.raw_get::<Value>(key)? {
        Value::Nil => Ok(None),
        Value::Integer(value) => Ok(Some(value as f64)),
        Value::Number(value) => Ok(Some(value)),
        other => Err(runtime_lua_error(format!(
            "field `{key}` must be a number when present, got {other:?}"
        ))),
    }
}

fn raw_bool_field(table: &Table, key: &str) -> mlua::Result<Option<bool>> {
    match table.raw_get::<Value>(key)? {
        Value::Nil => Ok(None),
        Value::Boolean(value) => Ok(Some(value)),
        other => Err(runtime_lua_error(format!(
            "field `{key}` must be a boolean when present, got {other:?}"
        ))),
    }
}

fn raw_usize_field(table: &Table, key: &str) -> mlua::Result<Option<usize>> {
    match table.raw_get::<Value>(key)? {
        Value::Nil => Ok(None),
        Value::Integer(value) if value >= 0 => Ok(Some(value as usize)),
        Value::Number(value) if value >= 0.0 && value.fract() == 0.0 => Ok(Some(value as usize)),
        other => Err(runtime_lua_error(format!(
            "field `{key}` must be a non-negative integer when present, got {other:?}"
        ))),
    }
}

fn raw_u32_field(table: &Table, key: &str) -> mlua::Result<Option<u32>> {
    match raw_usize_field(table, key)? {
        Some(value) => u32::try_from(value).map(Some).map_err(|_| {
            runtime_lua_error(format!(
                "field `{key}` must fit in a 32-bit unsigned integer when present"
            ))
        }),
        None => Ok(None),
    }
}

fn raw_index_field(table: &Table, key: &str) -> mlua::Result<Option<usize>> {
    match table.raw_get::<Value>(key)? {
        Value::Nil => Ok(None),
        Value::Integer(value) if value >= 0 => Ok(Some(value as usize)),
        Value::Number(value) if value >= 0.0 && value.fract() == 0.0 => Ok(Some(value as usize)),
        other => Err(runtime_lua_error(format!(
            "field `{key}` must be a non-negative integer when present, got {other:?}"
        ))),
    }
}

fn parse_label_tone(value: &str) -> mlua::Result<UiLabelTone> {
    parse_label_tone_token(value)
        .ok_or_else(|| runtime_lua_error(format!("unknown label tone `{value}`")))
}

fn parse_label_weight(value: &str) -> mlua::Result<UiLabelWeight> {
    parse_label_weight_token(value)
        .ok_or_else(|| runtime_lua_error(format!("unknown label weight `{value}`")))
}

fn parse_button_variant(value: &str) -> mlua::Result<UiButtonVariant> {
    parse_button_variant_token(value)
        .ok_or_else(|| runtime_lua_error(format!("unknown button variant `{value}`")))
}

fn parse_control_size(value: &str) -> mlua::Result<UiControlSize> {
    parse_control_size_token(value)
        .ok_or_else(|| runtime_lua_error(format!("unknown control size `{value}`")))
}

fn parse_number_input_axis(value: &str) -> mlua::Result<UiNumberInputAxis> {
    parse_number_input_axis_token(value)
        .ok_or_else(|| runtime_lua_error(format!("unknown number input axis `{value}`")))
}

fn parse_select_variant(value: &str) -> mlua::Result<UiSelectVariant> {
    parse_select_variant_token(value)
        .ok_or_else(|| runtime_lua_error(format!("unknown select variant `{value}`")))
}

fn parse_tabs_variant(value: &str) -> mlua::Result<UiTabsVariant> {
    parse_tabs_variant_token(value)
        .ok_or_else(|| runtime_lua_error(format!("unknown tabs variant `{value}`")))
}

fn parse_tooltip_placement(value: &str) -> mlua::Result<UiTooltipPlacement> {
    parse_tooltip_placement_token(value)
        .ok_or_else(|| runtime_lua_error(format!("unknown tooltip placement `{value}`")))
}

fn parse_skeleton_shape(value: &str) -> mlua::Result<UiSkeletonShape> {
    parse_skeleton_shape_token(value)
        .ok_or_else(|| runtime_lua_error(format!("unknown skeleton shape `{value}`")))
}

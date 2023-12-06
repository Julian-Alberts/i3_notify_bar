use crate::{
    component_manager::ManageComponents,
    property::{self, Properties},
    protocol::ClickEvent,
};

pub trait Padding {
    fn padding_left(&self) -> usize {
        self.padding().left
    }
    fn set_padding_left(&mut self, left: usize) {
        self.padding_mut().left = left;
    }
    fn padding_right(&self) -> usize {
        self.padding().right
    }
    fn set_padding_right(&mut self, right: usize) {
        self.padding_mut().right = right;
    }
    fn padding(&self) -> &property::Padding;
    fn padding_mut(&mut self) -> &mut property::Padding;
}
pub trait Text {
    fn full(&self) -> &str {
        &self.text().full
    }
    fn set_full(&mut self, full: String) {
        self.text_mut().full = full;
    }
    fn short(&self) -> Option<&str> {
        self.text().short.as_deref()
    }
    fn set_short(&mut self, short: Option<String>) {
        self.text_mut().short = short;
    }
    fn text(&self) -> &property::Text;
    fn text_mut(&mut self) -> &mut property::Text;
}
pub trait Color {
    fn color_text(&self) -> Option<&str> {
        self.color().text.as_deref()
    }
    fn set_color_text(&mut self, text: Option<String>) {
        self.color_mut().text = text;
    }
    fn color_background(&self) -> Option<&str> {
        self.color().background.as_deref()
    }
    fn set_color_background(&mut self, background: Option<String>) {
        self.color_mut().background = background;
    }
    fn color(&self) -> &property::Color;
    fn color_mut(&mut self) -> &mut property::Color;
}
pub trait Border {
    fn border_color(&self) -> Option<&str> {
        self.border().color.as_deref()
    }
    fn set_border_color(&mut self, color: Option<String>) {
        self.border_mut().color = color;
    }
    fn top(&self) -> Option<usize> {
        self.border().top
    }
    fn set_top(&mut self, top: Option<usize>) {
        self.border_mut().top = top;
    }
    fn right(&self) -> Option<usize> {
        self.border().right
    }
    fn set_right(&mut self, right: Option<usize>) {
        self.border_mut().right = right;
    }
    fn bottom(&self) -> Option<usize> {
        self.border().bottom
    }
    fn set_bottom(&mut self, bottom: Option<usize>) {
        self.border_mut().bottom = bottom;
    }
    fn left(&self) -> Option<usize> {
        self.border().left
    }
    fn set_left(&mut self, left: Option<usize>) {
        self.border_mut().left = left;
    }
    fn border(&self) -> &property::Border;
    fn border_mut(&mut self) -> &mut property::Border;
}
pub trait Separator {
    fn show(&self) -> bool {
        self.separator().show
    }
    fn set_show(&mut self, show: bool) {
        self.separator_mut().show = show;
    }
    fn block_width(&self) -> Option<usize> {
        self.separator().block_width
    }
    fn set_block_width(&mut self, block_width: Option<usize>) {
        self.separator_mut().block_width = block_width;
    }
    fn separator(&self) -> &property::Separator;
    fn separator_mut(&mut self) -> &mut property::Separator;
}
pub trait Align {
    fn align(&self) -> crate::property::Align;
    fn set_align(&mut self, align: crate::property::Align);
}
pub trait Markup {
    fn markup(&self) -> crate::property::Markup;
    fn set_markup(&mut self, markup: crate::property::Markup);
}
pub trait Urgent {
    fn urgent(&self) -> bool;
    fn set_urgent(&mut self, urgent: bool);
}
pub trait SimpleComponent: Component {
    fn properties(&self) -> &crate::property::Properties;
    fn properties_mut(&mut self) -> &mut crate::property::Properties;
    fn instance(&self) -> property::Instance {
        self.properties().instance
    }
}
pub trait Component {
    fn update(&mut self, dt: f64);
    fn event(&mut self, cm: &mut dyn ManageComponents, envnt: &ClickEvent);
    fn all_properties<'a>(&'a self) -> Box<dyn Iterator<Item = &Properties> + 'a>;
    fn name(&self) -> Option<&str> {
        None
    }
}

impl<T> Padding for T
where
    T: SimpleComponent,
{
    fn padding(&self) -> &property::Padding {
        &self.properties().padding
    }
    fn padding_mut(&mut self) -> &mut property::Padding {
        &mut self.properties_mut().padding
    }
}
impl<T> Text for T
where
    T: SimpleComponent,
{
    fn text(&self) -> &property::Text {
        &self.properties().text
    }
    fn text_mut(&mut self) -> &mut property::Text {
        &mut self.properties_mut().text
    }
}
impl<T> Color for T
where
    T: SimpleComponent,
{
    fn color(&self) -> &property::Color {
        &self.properties().color
    }
    fn color_mut(&mut self) -> &mut property::Color {
        &mut self.properties_mut().color
    }
}
impl<T> Separator for T
where
    T: SimpleComponent,
{
    fn separator(&self) -> &property::Separator {
        &self.properties().separator
    }
    fn separator_mut(&mut self) -> &mut property::Separator {
        &mut self.properties_mut().separator
    }
}
impl<T> Border for T
where
    T: SimpleComponent,
{
    fn border(&self) -> &property::Border {
        &self.properties().border
    }
    fn border_mut(&mut self) -> &mut property::Border {
        &mut self.properties_mut().border
    }
}
impl<T> Align for T
where
    T: SimpleComponent,
{
    fn align(&self) -> crate::property::Align {
        self.properties().align
    }
    fn set_align(&mut self, align: crate::property::Align) {
        self.properties_mut().align = align
    }
}
impl<T> Markup for T
where
    T: SimpleComponent,
{
    fn markup(&self) -> crate::property::Markup {
        self.properties().markup
    }
    fn set_markup(&mut self, markup: crate::property::Markup) {
        self.properties_mut().markup = markup
    }
}
impl<T> Urgent for T
where
    T: SimpleComponent,
{
    fn urgent(&self) -> bool {
        self.properties().urgent
    }
    fn set_urgent(&mut self, urgent: bool) {
        self.properties_mut().urgent = urgent
    }
}

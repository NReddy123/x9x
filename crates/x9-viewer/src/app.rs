use crate::image_view::{self, Decoded};
use gpui_kit::component::{
    button::{Button, ButtonVariants as _},
    input::{Input, InputState},
    ActiveTheme as _, Selectable as _, Sizable as _, Theme, ThemeMode,
};
use gpui_kit::{
    div, img, px, uniform_list, AppContext as _, ClickEvent, Context, Entity, FontWeight,
    InteractiveElement as _, IntoElement, ParentElement as _, PathPromptOptions, Render,
    ScrollStrategy, SharedString, StatefulInteractiveElement as _, Styled as _,
    UniformListScrollHandle, Window,
};
use gpui_kit::ImageSource;
use std::ops::Range;
use std::path::PathBuf;
use x9_core::{layout, FieldDef, ImageView, Item, X9File};

/// One line of the navigation tree.
#[derive(Clone)]
struct Row {
    depth: usize,
    label: SharedString,
    detail: SharedString,
    /// Record to select when clicked.
    record: usize,
    /// Item (index into `items`) the row belongs to, if any.
    item: Option<usize>,
}

struct Loaded {
    file: X9File,
    path: Option<PathBuf>,
    rows: Vec<Row>,
    items: Vec<Item>,
}

struct Shown {
    decoded: Result<Decoded, String>,
}

pub struct X9App {
    loaded: Option<Loaded>,
    dirty: bool,
    status: SharedString,
    selected_row: Option<usize>,
    selected_record: Option<usize>,
    images: Vec<ImageView>,
    image_ix: usize,
    shown: Option<Shown>,
    decode_seq: u64,
    zoom: f32,
    fields: Vec<(FieldDef, Entity<InputState>)>,
    scroll: UniformListScrollHandle,
    dark: bool,
}

fn money(cents: u64) -> String {
    format!("{}.{:02}", cents / 100, cents % 100)
}

impl X9App {
    pub fn new(path: Option<PathBuf>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut this = Self {
            loaded: None,
            dirty: false,
            status: "Open an X9 / X9.37 file to begin (Open button or pass a path on the command line)."
                .into(),
            selected_row: None,
            selected_record: None,
            images: vec![],
            image_ix: 0,
            shown: None,
            decode_seq: 0,
            zoom: 1.0,
            fields: vec![],
            scroll: UniformListScrollHandle::new(),
            dark: false,
        };
        if let Some(p) = path {
            this.load_path(p, window, cx);
        }
        this
    }

    fn load_path(&mut self, path: PathBuf, window: &mut Window, cx: &mut Context<Self>) {
        match std::fs::read(&path).map_err(|e| e.to_string()).and_then(|b| X9File::parse(&b).map_err(|e| e.to_string())) {
            Ok(file) => {
                self.status = format!(
                    "{} — {} records, {}",
                    path.display(),
                    file.records.len(),
                    file.encoding.label()
                )
                .into();
                self.set_file(file, Some(path), window, cx);
                self.dirty = false;
            }
            Err(e) => self.status = format!("Failed to open {}: {e}", path.display()).into(),
        }
        cx.notify();
    }

    fn set_file(&mut self, file: X9File, path: Option<PathBuf>, window: &mut Window, cx: &mut Context<Self>) {
        let (rows, items) = build_rows(&file);
        self.loaded = Some(Loaded { file, path, rows, items });
        self.selected_row = None;
        self.selected_record = None;
        self.images.clear();
        self.shown = None;
        self.fields.clear();
        if !self.loaded.as_ref().unwrap().rows.is_empty() {
            let first_item = self.loaded.as_ref().unwrap().rows.iter().position(|r| r.item.is_some()).unwrap_or(0);
            self.select_row(first_item, window, cx);
        }
    }

    fn select_row(&mut self, ix: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(l) = &self.loaded else { return };
        let Some(row) = l.rows.get(ix) else { return };
        self.selected_row = Some(ix);
        let rec = row.record;
        self.images = match row.item {
            Some(i) => l.file.images(l.items[i].range.clone()),
            None => vec![],
        };
        self.image_ix = 0;
        self.select_record(rec, window, cx);
        self.show_image(cx);
    }

    fn select_record(&mut self, rec: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(l) = &self.loaded else { return };
        self.selected_record = Some(rec);
        let defs = l.file.fields(rec);
        let vals: Vec<String> = defs.iter().map(|d| l.file.field_text(rec, d)).collect();
        self.fields = defs
            .iter()
            .zip(vals)
            .map(|(d, v)| {
                let st = cx.new(|cx| {
                    let mut s = InputState::new(window, cx);
                    s.set_value(v, window, cx);
                    s
                });
                (*d, st)
            })
            .collect();
        cx.notify();
    }

    /// Decode the current image on a background thread.
    fn show_image(&mut self, cx: &mut Context<Self>) {
        self.shown = None;
        let Some(l) = &self.loaded else { return };
        let Some(v) = self.images.get(self.image_ix) else { return cx.notify() };
        let bytes = l.file.image_bytes(v).to_vec();
        self.decode_seq += 1;
        let seq = self.decode_seq;
        cx.spawn(async move |this, cx| {
            let decoded = cx.background_spawn(async move { image_view::decode(&bytes) }).await;
            this.update(cx, |this, cx| {
                if this.decode_seq == seq {
                    this.shown = Some(Shown { decoded });
                    cx.notify();
                }
            })
            .ok();
        })
        .detach();
        cx.notify();
    }

    fn open_dialog(&mut self, _: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let rx = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some("Open X9 file".into()),
        });
        cx.spawn_in(_window, async move |this, cx| {
            if let Ok(Ok(Some(paths))) = rx.await {
                if let Some(p) = paths.into_iter().next() {
                    this.update_in(cx, |this, window, cx| this.load_path(p, window, cx)).ok();
                }
            }
        })
        .detach();
    }

    fn write_to(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        if let Some(l) = &mut self.loaded {
            match std::fs::write(&path, l.file.to_bytes()) {
                Ok(()) => {
                    self.status = format!("Saved {}", path.display()).into();
                    l.path = Some(path);
                    self.dirty = false;
                }
                Err(e) => self.status = format!("Save failed: {e}").into(),
            }
        }
        cx.notify();
    }

    fn save(&mut self, ev: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        match self.loaded.as_ref().and_then(|l| l.path.clone()) {
            Some(p) => self.write_to(p, cx),
            None => self.save_as(ev, window, cx),
        }
    }

    fn save_as(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        let dir = self.loaded.as_ref().and_then(|l| l.path.as_ref()?.parent().map(|p| p.to_path_buf())).unwrap_or_else(|| PathBuf::from("."));
        let rx = cx.prompt_for_new_path(&dir, Some("edited.x9"));
        cx.spawn_in(window, async move |this, cx| {
            if let Ok(Ok(Some(p))) = rx.await {
                this.update(cx, |this, cx| this.write_to(p, cx)).ok();
            }
        })
        .detach();
    }

    fn export_image(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        let Some(l) = &self.loaded else { return };
        let Some(v) = self.images.get(self.image_ix) else { return };
        let bytes = l.file.image_bytes(v).to_vec();
        let ext = if bytes.starts_with(&[0xFF, 0xD8]) { "jpg" } else { "tif" };
        let rx = cx.prompt_for_new_path(&PathBuf::from("."), Some(&format!("item.{ext}")));
        cx.spawn_in(window, async move |this, cx| {
            if let Ok(Ok(Some(p))) = rx.await {
                let msg = match std::fs::write(&p, bytes) {
                    Ok(()) => format!("Exported {}", p.display()),
                    Err(e) => format!("Export failed: {e}"),
                };
                this.update(cx, |this, cx| {
                    this.status = msg.into();
                    cx.notify();
                })
                .ok();
            }
        })
        .detach();
    }

    fn apply_fields(&mut self, _: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let (Some(rec), Some(l)) = (self.selected_record, &mut self.loaded) else { return };
        for (def, st) in &self.fields {
            let v = st.read(cx).value().to_string();
            if l.file.field_text(rec, def) != v {
                l.file.set_field(rec, def, &v);
            }
        }
        self.dirty = true;
        self.status = "Record updated".into();
        self.refresh_rows(cx);
    }

    fn recalc(&mut self, _: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(l) = &mut self.loaded {
            l.file.recalculate_controls();
            self.dirty = true;
            self.status = "Control totals recalculated (70 / 90 / 99)".into();
        }
        self.refresh_rows(cx);
    }

    fn validate(&mut self, _: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(l) = &self.loaded {
            let issues = l.file.validate();
            self.status = match issues.first() {
                None => "Validation passed: structure and control totals are consistent".into(),
                Some(i) => format!("{} issue(s). First: record {} — {}", issues.len(), i.record + 1, i.message).into(),
            };
        }
        cx.notify();
    }

    fn delete_item(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        let Some(ix) = self.selected_row else { return };
        let Some(l) = &mut self.loaded else { return };
        let Some(it) = l.rows[ix].item.map(|i| l.items[i].clone()) else { return };
        l.file.delete_item(&it);
        l.file.recalculate_controls();
        self.dirty = true;
        self.status = "Item deleted and control totals recalculated".into();
        let (rows, items) = build_rows(&l.file);
        l.rows = rows;
        l.items = items;
        let n = l.rows.len();
        if n > 0 {
            self.select_row(ix.min(n - 1), window, cx);
        }
        cx.notify();
    }

    fn refresh_rows(&mut self, cx: &mut Context<Self>) {
        if let Some(l) = &mut self.loaded {
            let (rows, items) = build_rows(&l.file);
            l.rows = rows;
            l.items = items;
        }
        cx.notify();
    }

    fn render_rows(&mut self, range: Range<usize>, cx: &mut Context<Self>) -> Vec<gpui_kit::AnyElement> {
        let Some(l) = &self.loaded else { return vec![] };
        let theme = cx.theme().clone();
        range
            .map(|ix| {
                let row = l.rows[ix].clone();
                let selected = self.selected_row == Some(ix);
                div()
                    .id(("row", ix))
                    .flex()
                    .flex_row()
                    .justify_between()
                    .items_center()
                    .h(px(26.))
                    .pl(px(8. + 14. * row.depth as f32))
                    .pr_2()
                    .cursor_pointer()
                    .text_sm()
                    .when(selected, |d| d.bg(theme.list_active))
                    .hover(|d| d.bg(theme.list_hover))
                    .when(row.item.is_none(), |d| d.font_weight(FontWeight::SEMIBOLD))
                    .on_click(cx.listener(move |this, _, window, cx| this.select_row(ix, window, cx)))
                    .child(row.label)
                    .child(div().text_color(theme.muted_foreground).child(row.detail))
                    .into_any_element()
            })
            .collect()
    }

    fn render_image(&self, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let theme = cx.theme().clone();
        let body = match &self.shown {
            Some(Shown { decoded: Ok(d) }) => {
                let w = d.width as f32 * self.zoom * 0.4;
                let h = d.height as f32 * self.zoom * 0.4;
                div()
                    .id("image-scroll")
                    .size_full()
                    .overflow_scroll()
                    .p_4()
                    .child(
                        img(ImageSource::Render(d.image.clone()))
                            .w(px(w))
                            .h(px(h))
                            .flex_none()
                            .border_1()
                            .border_color(theme.border)
                            .bg(gpui_kit::white()),
                    )
                    .into_any_element()
            }
            Some(Shown { decoded: Err(e) }) => centered(cx, format!("Cannot display image: {e}")),
            None if self.images.is_empty() => centered(cx, "No image for this selection".into()),
            None => centered(cx, "Decoding image…".into()),
        };
        let mut bar = div().flex().flex_row().gap_2().items_center().p_2().border_b_1().border_color(theme.border);
        for (i, v) in self.images.iter().enumerate() {
            bar = bar.child(
                Button::new(("side", i))
                    .small()
                    .label(if v.back { "Back" } else { "Front" })
                    .selected(i == self.image_ix)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.image_ix = i;
                        this.show_image(cx);
                    })),
            );
        }
        let info = match &self.shown {
            Some(Shown { decoded: Ok(d) }) => format!("{}×{} px  {:.0}%", d.width, d.height, self.zoom * 100.),
            _ => String::new(),
        };
        bar = bar
            .child(div().flex_1())
            .child(div().text_sm().text_color(theme.muted_foreground).child(info))
            .child(Button::new("zout").small().ghost().label("−").on_click(cx.listener(|this, _, _, cx| {
                this.zoom = (this.zoom / 1.25).max(0.2);
                cx.notify();
            })))
            .child(Button::new("zin").small().ghost().label("+").on_click(cx.listener(|this, _, _, cx| {
                this.zoom = (this.zoom * 1.25).min(8.0);
                cx.notify();
            })))
            .child(Button::new("zfit").small().ghost().label("100%").on_click(cx.listener(|this, _, _, cx| {
                this.zoom = 1.0;
                cx.notify();
            })))
            .child(Button::new("export").small().label("Export").on_click(cx.listener(Self::export_image)));
        div().flex().flex_col().size_full().child(bar).child(div().flex_1().min_h_0().child(body)).into_any_element()
    }

    fn render_inspector(&self, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let theme = cx.theme().clone();
        let mut col = div().flex().flex_col().size_full();
        let Some(l) = &self.loaded else { return col.into_any_element() };
        // Record tabs for the selected item.
        let mut tabs = div().flex().flex_row().flex_wrap().gap_1().p_2().border_b_1().border_color(theme.border);
        let range = self
            .selected_row
            .and_then(|r| l.rows[r].item)
            .map(|i| l.items[i].range.clone())
            .or_else(|| self.selected_record.map(|r| r..r + 1));
        if let Some(range) = range {
            for r in range {
                let t = l.file.rtype(r);
                tabs = tabs.child(
                    Button::new(("rec", r))
                        .small()
                        .label(format!("{t} {}", layout::record_name(&t)))
                        .selected(self.selected_record == Some(r))
                        .on_click(cx.listener(move |this, _, window, cx| this.select_record(r, window, cx))),
                );
            }
        }
        col = col.child(tabs);
        let mut form = div().id("fields").flex().flex_col().gap_2().p_3().flex_1().min_h_0().overflow_y_scroll();
        for (def, st) in &self.fields {
            form = form.child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div().text_xs().text_color(theme.muted_foreground).child(format!(
                            "{} · pos {}–{}",
                            def.name,
                            def.start,
                            def.start + def.len - 1
                        )),
                    )
                    .child(Input::new(st).small()),
            );
        }
        if self.fields.is_empty() && self.selected_record.is_some() {
            form = form.child(div().text_sm().text_color(theme.muted_foreground).child("No field layout for this record type."));
        }
        col = col.child(form);
        if let Some(rec) = self.selected_record {
            let raw = l.file.records[rec].text(l.file.encoding, 1, 160);
            col = col.child(
                div()
                    .p_2()
                    .border_t_1()
                    .border_color(theme.border)
                    .font_family("monospace")
                    .text_xs()
                    .text_color(theme.muted_foreground)
                    .child(raw),
            );
        }
        col.child(
            div()
                .flex()
                .flex_row()
                .gap_2()
                .p_2()
                .border_t_1()
                .border_color(theme.border)
                .child(Button::new("apply").primary().small().label("Apply").on_click(cx.listener(Self::apply_fields)))
                .child(Button::new("recalc").small().label("Recalc totals").on_click(cx.listener(Self::recalc)))
                .child(Button::new("delitem").small().label("Delete item").on_click(cx.listener(Self::delete_item))),
        )
        .into_any_element()
    }
}

fn centered(cx: &mut Context<X9App>, msg: String) -> gpui_kit::AnyElement {
    div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .text_color(cx.theme().muted_foreground)
        .child(msg)
        .into_any_element()
}

fn build_rows(file: &X9File) -> (Vec<Row>, Vec<Item>) {
    let enc = file.encoding;
    let mut rows = Vec::new();
    let mut items = Vec::new();
    rows.push(Row {
        depth: 0,
        label: "File Header".into(),
        detail: file.records[0].text(enc, 37, 18).trim().to_string().into(),
        record: 0,
        item: None,
    });
    for (ci, cl) in file.outline().iter().enumerate() {
        rows.push(Row {
            depth: 1,
            label: format!("Cash Letter {}", ci + 1).into(),
            detail: file.records[cl.header].text(enc, 45, 8).trim().to_string().into(),
            record: cl.header,
            item: None,
        });
        for (bi, b) in cl.bundles.iter().enumerate() {
            rows.push(Row {
                depth: 2,
                label: format!("Bundle {}", bi + 1).into(),
                detail: format!("{} items", b.items.len()).into(),
                record: b.header,
                item: None,
            });
            for it in &b.items {
                let r = it.range.start;
                let amount = file.amount(r);
                let seq = file.records[r].text(enc, 58, 15).trim().to_string();
                items.push(it.clone());
                rows.push(Row {
                    depth: 3,
                    label: format!("{} {}", if it.is_credit { "Credit" } else { "Check" }, seq).into(),
                    detail: money(amount).into(),
                    record: r,
                    item: Some(items.len() - 1),
                });
            }
        }
    }
    (rows, items)
}

use gpui_kit::prelude::FluentBuilder as _;

impl Render for X9App {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let count = self.loaded.as_ref().map_or(0, |l| l.rows.len());
        let title = self
            .loaded
            .as_ref()
            .and_then(|l| l.path.as_ref())
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "X9 Viewer".into());
        let toolbar = div()
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            .px_3()
            .h(px(44.))
            .border_b_1()
            .border_color(theme.border)
            .bg(theme.title_bar)
            .child(div().font_weight(FontWeight::BOLD).child(format!("{title}{}", if self.dirty { " •" } else { "" })))
            .child(div().flex_1())
            .child(Button::new("open").primary().small().label("Open").on_click(cx.listener(Self::open_dialog)))
            .child(Button::new("save").small().label("Save").on_click(cx.listener(Self::save)))
            .child(Button::new("saveas").small().label("Save As").on_click(cx.listener(Self::save_as)))
            .child(Button::new("validate").small().label("Validate").on_click(cx.listener(Self::validate)))
            .child(Button::new("theme").small().ghost().label(if self.dark { "Light" } else { "Dark" }).on_click(
                cx.listener(|this, _, window, cx| {
                    this.dark = !this.dark;
                    Theme::change(if this.dark { ThemeMode::Dark } else { ThemeMode::Light }, Some(window), cx);
                    cx.notify();
                }),
            ));
        let tree = div()
            .w(px(320.))
            .h_full()
            .flex_none()
            .border_r_1()
            .border_color(theme.border)
            .bg(theme.sidebar)
            .child(
                uniform_list(
                    "x9-rows",
                    count,
                    cx.processor(|this, range, _window, cx| this.render_rows(range, cx)),
                )
                .track_scroll(&self.scroll)
                .size_full(),
            );
        let _ = ScrollStrategy::Nearest;
        let image = self.render_image(cx);
        let inspector = self.render_inspector(cx);
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.background)
            .text_color(theme.foreground)
            .child(toolbar)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_1()
                    .min_h_0()
                    .child(tree)
                    .child(div().flex_1().min_w_0().h_full().bg(theme.muted).child(image))
                    .child(
                        div().w(px(380.)).h_full().flex_none().border_l_1().border_color(theme.border).child(inspector),
                    ),
            )
            .child(
                div()
                    .h(px(26.))
                    .px_3()
                    .flex()
                    .items_center()
                    .text_xs()
                    .text_color(theme.muted_foreground)
                    .border_t_1()
                    .border_color(theme.border)
                    .child(self.status.clone()),
            )
    }
}

#![windows_subsystem = "windows"]
use eframe::egui;
use windows::{core::*, Win32::{Media::Audio::*, Media::Audio::Endpoints::IAudioEndpointVolume, System::Com::*, Devices::FunctionDiscovery::PKEY_Device_FriendlyName}};
struct Device { id:String, name:String, volume:IAudioEndpointVolume }
struct Driver { output:Vec<Device>, input:Vec<Device>, out:usize, mic:usize, status:String }
fn devices(flow:EDataFlow)->Result<Vec<Device>> { unsafe {
 let e:IMMDeviceEnumerator=CoCreateInstance(&MMDeviceEnumerator,None,CLSCTX_ALL)?;
 let c=e.EnumAudioEndpoints(flow,DEVICE_STATE_ACTIVE)?; let mut result=Vec::new();
 for i in 0..c.GetCount()? { let d=c.Item(i)?; let p=d.GetId()?; let id=p.to_string()?; CoTaskMemFree(Some(p.0 as _)); let props=d.OpenPropertyStore(STGM_READ)?; let name=props.GetValue(&PKEY_Device_FriendlyName)?.to_string(); let volume:IAudioEndpointVolume=d.Activate(CLSCTX_ALL,None)?; result.push(Device{id,name,volume}); } Ok(result)
}}
fn default_id(flow:EDataFlow)->Option<String>{unsafe {let e:IMMDeviceEnumerator=CoCreateInstance(&MMDeviceEnumerator,None,CLSCTX_ALL).ok()?; let d=e.GetDefaultAudioEndpoint(flow,eMultimedia).ok()?; let p=d.GetId().ok()?; let id=p.to_string().ok(); CoTaskMemFree(Some(p.0 as _)); id}}
impl Driver {fn new()->Self {unsafe {let _=CoInitializeEx(None,COINIT_APARTMENTTHREADED);} let mut a=Self{output:vec![],input:vec![],out:0,mic:0,status:String::new()}; a.refresh(); a}
fn refresh(&mut self){let o=self.output.get(self.out).map(|d|d.id.clone()).or_else(||default_id(eRender)); let m=self.input.get(self.mic).map(|d|d.id.clone()).or_else(||default_id(eCapture)); match (devices(eRender),devices(eCapture)){(Ok(a),Ok(b))=>{self.output=a; self.input=b; self.out=o.and_then(|id|self.output.iter().position(|d|d.id==id)).unwrap_or(0); self.mic=m.and_then(|id|self.input.iter().position(|d|d.id==id)).unwrap_or(0); self.status="Choose your USB device from each list. Changes apply immediately.".into();},(Err(e),_)|(_,Err(e))=>self.status=format!("Could not load devices: {e}")}}}

fn section(ui: &mut egui::Ui, title: &str, subtitle: &str, devices: &[Device], selected: &mut usize, status: &mut String) {
    let panel = egui::Frame::new().fill(egui::Color32::from_rgb(43, 43, 47)).corner_radius(12).inner_margin(egui::Margin::same(20));
    panel.show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new(title).size(18.0).strong().color(egui::Color32::WHITE));
                ui.label(egui::RichText::new(subtitle).size(12.0).color(egui::Color32::from_gray(175)));
            });
            if let Some(device) = devices.get(*selected) { unsafe {
                let muted = device.volume.GetMute().map(|m| m.as_bool()).unwrap_or(false);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(egui::Button::new(if muted { "Unmute" } else { "Mute" }).corner_radius(7).min_size(egui::vec2(70.0, 30.0))).clicked() {
                        if let Err(e) = device.volume.SetMute(!muted, std::ptr::null()) { *status = format!("Could not change mute: {e}"); }
                    }
                });
            }}
        });
        ui.add_space(12.0);
        egui::ComboBox::from_id_salt(title).selected_text(devices.get(*selected).map(|d| d.name.as_str()).unwrap_or("No device connected"))
            .width(ui.available_width() - 22.0).show_ui(ui, |ui| { for (i, d) in devices.iter().enumerate() { ui.selectable_value(selected, i, &d.name); } });
        ui.add_space(16.0);
        if let Some(device) = devices.get(*selected) { unsafe {
            match device.volume.GetMasterVolumeLevelScalar() {
                Ok(value) => {
                    let mut level = (value * 100.0).round();
                    ui.horizontal(|ui| {
                        let width = (ui.available_width() - 58.0).max(100.0);
                        ui.spacing_mut().slider_width = width;
                        let slider = egui::Slider::new(&mut level, 0.0..=100.0).show_value(false).trailing_fill(true).handle_shape(egui::style::HandleShape::Rect { aspect_ratio: 1.25 });
                        let response = ui.add(slider);
                        let x = egui::lerp((response.rect.left() + response.rect.height() * 0.5)..=(response.rect.right() - response.rect.height() * 0.5), level / 100.0);
                        let y = response.rect.center().y;
                        let stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_gray(195));
                        for dx in [-2.0, 2.0] { ui.painter().line_segment([egui::pos2(x + dx, y - 4.0), egui::pos2(x + dx, y + 4.0)], stroke); }
                        if response.changed() {
                            if let Err(e) = device.volume.SetMasterVolumeLevelScalar(level / 100.0, std::ptr::null()) { *status = format!("Could not change volume: {e}"); }
                        }
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new(format!("{level:.0}%")).size(16.0).strong());
                    });
                }
                Err(e) => *status = format!("Could not read volume: {e}"),
            }
        }} else { ui.label("Connect a device and select Refresh."); }
    });
}
impl eframe::App for Driver {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        ctx.request_repaint_after(std::time::Duration::from_millis(1000));
        egui::CentralPanel::default().frame(egui::Frame::new().fill(egui::Color32::from_rgb(30,30,33)).inner_margin(egui::Margin::same(24))).show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("SoundTune").size(27.0).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(egui::Button::new("Refresh").corner_radius(7).min_size(egui::vec2(75.0,32.0))).clicked() { self.refresh(); }
                });
            });
            ui.label(egui::RichText::new("Control your sound").size(13.0).color(egui::Color32::from_gray(175)));
            ui.add_space(20.0);
            section(ui,"Output","Earphones and speakers",&self.output,&mut self.out,&mut self.status);
            ui.add_space(12.0);
            section(ui,"Microphone","Input level",&self.input,&mut self.mic,&mut self.status);
            ui.add_space(14.0);
            ui.label(egui::RichText::new(&self.status).size(12.0).color(egui::Color32::from_gray(170)));
        });
    }
}
fn main()->eframe::Result {
    let options=eframe::NativeOptions{viewport:egui::ViewportBuilder::default().with_icon(egui::IconData { rgba: include_bytes!("../assets/icon.rgba").to_vec(), width: 256, height: 256 }).with_inner_size([580.0,520.0]).with_min_inner_size([480.0,500.0]),..Default::default()};
    eframe::run_native("SoundTune",options,Box::new(|cc| {
        let mut v=egui::Visuals::dark();
        v.panel_fill=egui::Color32::from_rgb(30,30,33);
        v.selection.bg_fill=egui::Color32::from_rgb(99,174,245);
        v.widgets.active.bg_fill=egui::Color32::from_rgb(99,174,245);
        v.widgets.inactive.corner_radius=egui::CornerRadius::same(6);
        v.widgets.hovered.corner_radius=egui::CornerRadius::same(6);
        v.widgets.active.corner_radius=egui::CornerRadius::same(6);
        cc.egui_ctx.set_visuals(v);
        Ok(Box::new(Driver::new()))
    }))
}






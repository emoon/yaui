use crate::{Ui, area, rgb, rgba};
use clay_layout::{
    color::Color as ClayColor, fixed, grow, layout::LayoutDirection, layout::Padding,
};

// DAW-specific data structures
#[derive(Debug, Clone)]
pub struct Track {
    pub name: String,
    pub color: ClayColor,
    pub muted: bool,
    pub soloed: bool,
    pub volume: f32,
    pub pan: f32,
    pub clips: Vec<Clip>,
    pub track_type: TrackType,
}

#[derive(Debug, Clone)]
pub enum TrackType {
    Audio,
    Midi,
    Instrument,
    Bus,
}

#[derive(Debug, Clone)]
pub struct Clip {
    pub name: String,
    pub start_time: f32,
    pub duration: f32,
    pub color: ClayColor,
    pub clip_type: ClipType,
}

#[derive(Debug, Clone)]
pub enum ClipType {
    Audio { waveform_data: Vec<f32> },
    Midi { notes: Vec<MidiNote> },
}

#[derive(Debug, Clone)]
pub struct MidiNote {
    pub pitch: u8,
    pub velocity: u8,
    pub start: f32,
    pub duration: f32,
}

#[derive(Debug)]
pub struct DawState {
    pub tracks: Vec<Track>,
    pub timeline_position: f32,
    pub zoom_level: f32,
    pub is_playing: bool,
    pub is_recording: bool,
    pub tempo: f32,
    pub time_signature: (u8, u8),
    pub selected_tool: Tool,
    pub mixer_visible: bool,
    // String storage to keep formatted strings alive
    pub time_display_text: String,
    pub track_volume_texts: Vec<String>,
    pub timeline_marker_texts: Vec<String>,
    pub piano_key_ids: Vec<String>,
    pub clip_ids: Vec<String>,
    pub track_row_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum Tool {
    Select,
    Draw,
    Erase,
    Move,
    Cut,
    Zoom,
}

impl DawState {
    pub fn update_time_display(&mut self) {
        let minutes = (self.timeline_position / 60.0) as u32;
        let seconds = (self.timeline_position % 60.0) as u32;
        let milliseconds = ((self.timeline_position % 1.0) * 1000.0) as u32;
        self.time_display_text = format!("{:02}:{:02}.{:03}", minutes, seconds, milliseconds);
    }

    pub fn update_track_volume_text(&mut self, track_idx: usize) {
        if track_idx < self.tracks.len() && track_idx < self.track_volume_texts.len() {
            self.track_volume_texts[track_idx] =
                format!("Vol: {:.1}", self.tracks[track_idx].volume);
        }
    }
}

impl Default for DawState {
    fn default() -> Self {
        let tracks = vec![
            Track {
                name: "Bongos".to_string(),
                color: rgb(255, 100, 150),
                muted: false,
                soloed: false,
                volume: 0.8,
                pan: 0.0,
                clips: vec![],
                track_type: TrackType::Audio,
            },
            Track {
                name: "Congas".to_string(),
                color: rgb(255, 120, 180),
                muted: false,
                soloed: false,
                volume: 0.7,
                pan: 0.2,
                clips: vec![],
                track_type: TrackType::Audio,
            },
            Track {
                name: "Cowbells".to_string(),
                color: rgb(255, 140, 210),
                muted: false,
                soloed: false,
                volume: 0.6,
                pan: -0.1,
                clips: vec![],
                track_type: TrackType::Audio,
            },
        ];

        // Pre-allocate string storage
        let track_volume_texts = tracks
            .iter()
            .map(|track| format!("Vol: {:.1}", track.volume))
            .collect();
        let timeline_marker_texts = (0..20).map(|i| format!("{}", i)).collect();
        let mut piano_key_ids = Vec::new();
        for octave in 0..4 {
            for note in 0..12 {
                piano_key_ids.push(format!("key_{}_{}", octave, note));
            }
        }

        // Pre-allocate clip and track row IDs (for a reasonable number)
        let clip_ids: Vec<String> = (0..100).map(|i| format!("clip_{}", i)).collect();
        let track_row_ids: Vec<String> = (0..20).map(|i| format!("track_row_{}", i)).collect();

        let mut state = Self {
            tracks,
            timeline_position: 0.0,
            zoom_level: 1.0,
            is_playing: false,
            is_recording: false,
            tempo: 120.0,
            time_signature: (4, 4),
            selected_tool: Tool::Select,
            mixer_visible: true,
            time_display_text: String::new(),
            track_volume_texts,
            timeline_marker_texts,
            piano_key_ids,
            clip_ids,
            track_row_ids,
        };

        // Initialize time display text
        state.update_time_display();
        state
    }
}

// Reusable UI components that should be added to the base UI library
trait UiExtensions {
    fn knob(&self, label: &str, value: &mut f32, min: f32, max: f32) -> bool;
    fn fader(&self, label: &str, value: &mut f32, vertical: bool) -> bool;
    fn waveform_display(&self, data: &[f32], width: f32, height: f32, color: ClayColor);
    fn piano_roll(&self, notes: &[MidiNote], width: f32, height: f32);
    fn meter(&self, level: f32, peak: f32, vertical: bool);
    fn transport_button(&self, icon: &str, active: bool) -> bool;
    fn track_header(&self, track: &Track) -> TrackHeaderResponse;
}

// Note: The area! macro replaces the need for ui.rect() and ui.with_layout()
// The macro provides a cleaner, more declarative syntax while hiding Clay implementation details

#[derive(Debug)]
pub struct TrackHeaderResponse {
    pub mute_clicked: bool,
    pub solo_clicked: bool,
    pub volume_changed: Option<f32>,
    pub pan_changed: Option<f32>,
}

// Top toolbar components
fn toolbar_parameter_controls(ui: &Ui) {
    area!(ui, {
        id: "toolbar_parameter_controls",
        layout: {
            width: fixed!(200.0),
            height: fixed!(40.0),
            padding: Padding::all(5),
            direction: LayoutDirection::LeftToRight,
        },
        background_color: rgb(32, 32, 32),
    }, |ui: &Ui| {
        ui.label("Parameter", rgb(200, 200, 200));
        ui.label("Control", rgb(200, 200, 200));
    });
}

fn toolbar_tools(state: &mut DawState, ui: &Ui) {
    area!(ui, {
        id: "toolbar_tools",
        layout: {
            width: fixed!(300.0),
            height: fixed!(40.0),
            padding: Padding::all(2),
            direction: LayoutDirection::LeftToRight,
        },
    }, |ui: &Ui| {
        let tools = [
            (Tool::Select, "🔍oesthu"),
            (Tool::Draw, "✏️oesth"),
            (Tool::Erase, "🗑️osteh"),
            (Tool::Move, "↔️osethu"),
            (Tool::Cut, "✂️oust"),
            (Tool::Zoom, "🔍oesuth"),
        ];

        for (_tool, icon) in tools {
            let is_selected = matches!(state.selected_tool, _tool);
            ui.label(icon, if is_selected {
                rgb(100, 150, 255)
            } else {
                rgba(150, 150, 150, 128) // Semi-transparent when not selected
            });
        }
    });
}

fn transport_controls(state: &mut DawState, ui: &Ui) {
    area!(ui, {
        id: "transport_controls",
        layout: {
            width: fixed!(250.0),
            height: fixed!(40.0),
            padding: Padding::all(5),
            direction: LayoutDirection::LeftToRight,
        },
    }, |ui: &Ui| {
        ui.label("⏮️", rgb(200, 200, 200)); // Previous
        ui.label("⏹️", rgb(200, 200, 200)); // Stop
        ui.label(if state.is_playing { "⏸️" } else { "▶️" },
                if state.is_playing { rgb(100, 255, 100) } else { rgb(200, 200, 200) });
        ui.label("⏭️", rgb(200, 200, 200)); // Next
        ui.label(if state.is_recording { "⏺️" } else { "⏺️" },
                if state.is_recording { rgb(255, 100, 100) } else { rgb(200, 200, 200) });
    });
}

fn time_display(state: &DawState, ui: &Ui) {
    area!(ui, {
        id: "time_display",
        layout: {
            width: fixed!(150.0),
            height: fixed!(40.0),
            padding: Padding::all(10),
        },
        background_color: rgb(20, 20, 20),
    }, |ui: &Ui| {
        ui.label(&state.time_display_text, rgb(100, 255, 100));
    });
}

fn toolbar(state: &mut DawState, ui: &Ui) {
    area!(ui, {
        id: "toolbar",
        layout: {
            width: grow!(),
            height: fixed!(50.0),
            padding: Padding::all(5),
            direction: LayoutDirection::LeftToRight,
        },
        background_color: rgb(40, 40, 40),
    }, |ui| {
        toolbar_parameter_controls(ui);
        toolbar_tools(state, ui);
        transport_controls(state, ui);
        time_display(state, ui);
    });
}

// Track area components
fn track_header(track: &Track, ui: &Ui) {
    area!(ui, {
        id: "track_header",
        layout: {
            width: fixed!(200.0),
            height: fixed!(80.0),
            padding: Padding::all(5),
            direction: LayoutDirection::TopToBottom,
        },
        background_color: rgb(50, 50, 50),
    }, |ui: &Ui| {
        ui.label(&track.name, rgb(255, 255, 255));

        area!(ui, {
            id: "track_controls",
            layout: {
                width: grow!(),
                height: fixed!(30.0),
                direction: LayoutDirection::LeftToRight,
            },
        }, |ui: &Ui| {
            ui.label("M", if track.muted { rgb(255, 100, 100) } else { rgb(100, 100, 100) });
            ui.label("S", if track.soloed { rgb(255, 255, 100) } else { rgb(100, 100, 100) });
            // Note: For now using a static string, would need track index to use stored volume text
            ui.label("Vol: N/A", rgb(200, 200, 200));
        });
    });
}

fn track_timeline(track: &Track, timeline_width: f32, state: &DawState, ui: &Ui) {
    area!(ui, {
        id: "track_timeline",
        layout: {
            width: fixed!(timeline_width),
            height: fixed!(80.0),
        },
        background_color: track.color,
    }, |ui: &Ui| {
        for (clip_idx, clip) in track.clips.iter().enumerate() {
            let _clip_x = clip.start_time * 50.0; // 50 pixels per second
            let clip_width = clip.duration * 50.0;

            let clip_id = if clip_idx < state.clip_ids.len() {
                &state.clip_ids[clip_idx]
            } else {
                "default_clip"
            };
            area!(ui, {
                id: clip_id,
                layout: {
                    width: fixed!(clip_width),
                    height: fixed!(60.0),
                    padding: Padding::all(2),
                },
                background_color: clip.color,
            }, |ui: &Ui| {
                ui.label(&clip.name, rgb(255, 255, 255));

                match &clip.clip_type {
                    ClipType::Audio { waveform_data: _ } => {
                        // Render waveform visualization
                    },
                    ClipType::Midi { notes: _ } => {
                        // Render MIDI notes visualization
                    },
                }
            });
        }
    });
}

fn track_area(state: &DawState, ui: &Ui) {
    let _timeline_width = 1200.0; // Should be based on zoom and project length

    area!(ui, {
        id: "track_area",
        layout: {
            width: grow!(),
            height: fixed!(600.0),
            direction: LayoutDirection::TopToBottom,
        },
        background_color: rgb(0, 0, 255),
    }, |ui| {
        // Time ruler
        area!(ui, {
            id: "time_ruler",
            layout: {
                width: grow!(),
                height: fixed!(30.0),
            },
            background_color: rgb(60, 60, 60),
        }, |ui: &Ui| {
            for i in 0..20 {
                let _x = i as f32 * 60.0; // Every second
                if i < state.timeline_marker_texts.len() {
                    ui.label(&state.timeline_marker_texts[i], rgb(200, 200, 200));
                }
            }
        });

        /*
        for (track_idx, track) in state.tracks.iter().enumerate() {
            let track_row_id = if track_idx < state.track_row_ids.len() {
                &state.track_row_ids[track_idx]
            } else {
                "default_track_row"
            };
            area!(ui, {
                id: track_row_id,
                layout: {
                    width: grow!(),
                    height: fixed!(80.0),
                    direction: LayoutDirection::LeftToRight,
                },
            }, |ui| {
                track_header(track, ui);
                track_timeline(track, timeline_width, state, ui);
            });
        }

         */
    });
}

// Mixer panel components
fn channel_strip(track: &Track, ui: &Ui) {
    area!(ui, {
        id: "channel_strip",
        layout: {
            width: fixed!(80.0),
            height: fixed!(400.0),
            padding: Padding::all(5),
            direction: LayoutDirection::TopToBottom,
        },
        background_color: rgb(45, 45, 45),
    }, |ui: &Ui| {
        ui.label(&track.name, rgb(255, 255, 255));

        // EQ section
        area!(ui, {
            id: "eq_section",
            layout: {
                width: grow!(),
                height: fixed!(100.0),
            },
            background_color: rgb(35, 35, 35),
        }, |ui: &Ui| {
            ui.label("EQ", rgb(150, 150, 150));
        });

        // Effects section
        area!(ui, {
            id: "fx_section",
            layout: {
                width: grow!(),
                height: fixed!(150.0),
            },
            background_color: rgb(40, 40, 40),
        }, |ui: &Ui| {
            ui.label("FX", rgb(150, 150, 150));
        });

        // Fader and controls
        area!(ui, {
            id: "fader_controls",
            layout: {
                width: grow!(),
                height: grow!(),
                direction: LayoutDirection::TopToBottom,
            },
        }, |ui: &Ui| {
            // For now using static text - would need track index and state to use stored volume text
            ui.label("0.8", rgb(200, 200, 200));

            // Volume fader (vertical)
            area!(ui, {
                id: "volume_fader",
                layout: {
                    width: fixed!(20.0),
                    height: grow!(),
                },
                background_color: rgb(60, 60, 60),
            }, |_ui| {
                // Fader handle
                let _handle_y = (1.0 - track.volume) * 100.0;
            });

            // Mute/Solo buttons
            area!(ui, {
                id: "mute_solo_buttons",
                layout: {
                    width: grow!(),
                    height: fixed!(30.0),
                    direction: LayoutDirection::LeftToRight,
                },
            }, |ui: &Ui| {
                ui.label("M", if track.muted { rgb(255, 100, 100) } else { rgb(100, 100, 100) });
                ui.label("S", if track.soloed { rgb(255, 255, 100) } else { rgb(100, 100, 100) });
            });
        });
    });
}

fn mixer_panel(state: &DawState, ui: &Ui) {
    if !state.mixer_visible {
        return;
    }

    area!(ui, {
        id: "mixer_panel",
        layout: {
            width: fixed!(400.0),
            height: grow!(),
            padding: Padding::all(5),
            direction: LayoutDirection::LeftToRight,
        },
        background_color: rgb(50, 50, 50),
    }, |ui| {
        /*
        for track in &state.tracks {
            channel_strip(track, ui);
        }

         */

        // Master section
        area!(ui, {
            id: "master_section",
            layout: {
                width: fixed!(100.0),
                height: grow!(),
            },
            background_color: rgb(60, 50, 50),
        }, |ui: &Ui| {
            ui.label("MASTER", rgb(255, 255, 255));
        });
    });
}

// Piano roll / step sequencer at bottom
fn piano_roll_panel(state: &DawState, ui: &Ui) {
    area!(ui, {
        id: "piano_roll_panel",
        layout: {
            width: grow!(),
            height: fixed!(200.0),
            direction: LayoutDirection::LeftToRight,
        },
        background_color: rgb(30, 30, 40),
    }, |ui: &Ui| {
        // Piano keys
        area!(ui, {
            id: "piano_keys",
            layout: {
                width: fixed!(80.0),
                height: grow!(),
            },
            background_color: rgb(25, 25, 35),
        }, |ui: &Ui| {
            for octave in 0..4 {
                for note in 0..12 {
                    let is_black_key = matches!(note, 1 | 3 | 6 | 8 | 10);
                    let key_color = if is_black_key {
                        rgb(20, 20, 20)
                    } else {
                        rgb(240, 240, 240)
                    };

                    let key_index = octave * 12 + note;
                    let id = if key_index < state.piano_key_ids.len() {
                        &state.piano_key_ids[key_index]
                    } else {
                        "default_key"
                    };

                    area!(ui, {
                        id: id,
                        layout: {
                            width: grow!(),
                            height: fixed!(12.0),
                        },
                        background_color: key_color,
                    }, |_ui| {});
                }
            }
        });

        // Note grid
        area!(ui, {
            id: "note_grid",
            layout: {
                width: grow!(),
                height: grow!(),
            },
            background_color: rgb(35, 35, 45),
        }, |_ui| {
            // Grid lines and notes would be drawn here
        });
    });
}

fn impact_panel(_state: &DawState, ui: &Ui) {
    area!(ui, {
        id: "impact_panel",
        layout: {
            width: grow!(),
            height: grow!(),
            direction: LayoutDirection::TopToBottom,
        },
        background_color: rgb(50, 150, 60),
    }, |_ui| {});
}

fn mixing_panel(state: &DawState, ui: &Ui) {
    area!(ui, {
        id: "mixing_panel",
        layout: {
            width: grow!(),
            height: grow!(),
            direction: LayoutDirection::TopToBottom,
        },
        background_color: rgb(160, 60, 70),
    }, |_ui| {});
}

fn panels(state: &DawState, ui: &Ui) {
    area!(ui, {
        id: "panels",
        layout: {
            width: fixed!(400.0),
            height: grow!(),
            direction: LayoutDirection::TopToBottom,
        },
        background_color: rgb(40, 40, 50),
    }, |ui| {
        impact_panel(&state, &ui);
        mixing_panel(&state, &ui);
    });
}

fn playback_toolbar(state: &DawState, ui: &Ui) {
    area!(ui, {
        id: "playback_toolbar",
        layout: {
            width: grow!(),
            height: fixed!(80.0),
        },
        background_color: rgb(40, 40, 150),
    }, |ui| {
    });
}

pub fn daw_ui(state: &mut DawState, ui: &Ui, width: f32, height: f32) {
    // Update time display (simulate time progression)
    state.timeline_position += 0.1; // Simulate time passing
    state.update_time_display();

    area!(ui, {
        id: "daw_ui_root",
        layout: {
            width: fixed!(width),
            height: fixed!(height),
            direction: LayoutDirection::TopToBottom,
        },
    }, |ui| {
        // Top toolbar
        toolbar(state, ui);

        // Main content area
        area!(ui, {
            id: "main_content",
            layout: {
                width: grow!(),
                height: grow!(),
                direction: LayoutDirection::LeftToRight,
            },
        }, |ui| {
            // Track area (left/center)
            panels(&state, ui);
        });

       playback_toolbar(state, ui);

        // Bottom piano roll/step sequencer
        //piano_roll_panel(&state, ui);
    });
}

/*
SUGGESTIONS FOR GENERIC UI LIBRARY EXTENSIONS:

1. Layout Builder Pattern Extensions:
   - ui.rect() -> Declaration builder (IMPLEMENTED ABOVE)
   - .width(), .height(), .background_color(), .direction(), .padding() chainable methods
   - .child_alignment(), .child_gap(), .border(), .corner_radius()

2. Interactive Controls (like imgui):
   - ui.button(text) -> ButtonResponse { clicked: bool, hovered: bool }
   - ui.checkbox(text, &mut bool) -> bool (changed)
   - ui.slider(text, &mut f32, min, max) -> bool (changed)
   - ui.drag_float(text, &mut f32, speed) -> bool (changed)
   - ui.input_text(text, &mut String) -> bool (changed)
   - ui.dropdown(text, &[&str], &mut usize) -> bool (changed)

3. Audio/Media Specific Controls:
   - ui.knob(text, &mut f32, min, max) -> ControlResponse
   - ui.fader(text, &mut f32, vertical: bool) -> ControlResponse
   - ui.meter(level: f32, peak: f32, vertical: bool)
   - ui.waveform_display(&[f32], width, height, color)
   - ui.spectrum_analyzer(&[f32], width, height)
   - ui.transport_controls(&mut TransportState) -> TransportResponse

4. Layout Helpers:
   - ui.horizontal(|ui| { ... }) - implicit LeftToRight layout
   - ui.vertical(|ui| { ... }) - implicit TopToBottom layout
   - ui.group(|ui| { ... }) - creates isolated layout group
   - ui.separator() - visual separator line
   - ui.spacing(amount) - add fixed spacing

5. Drawing Primitives:
   - ui.rect_filled(rect, color)
   - ui.rect_stroke(rect, color, thickness)
   - ui.circle_filled(center, radius, color)
   - ui.line(start, end, color, thickness)
   - ui.path(&[Point], color, thickness, filled: bool)

6. Mouse/Input Handling:
   - ui.is_item_hovered() -> bool
   - ui.is_item_clicked() -> bool
   - ui.mouse_pos() -> Option<(f32, f32)>
   - ui.drag_delta() -> (f32, f32)
   - Response types should include interaction state

7. Styling System:
   - ui.push_style_var(StyleVar, value)
   - ui.pop_style_var()
   - Style override system for colors, spacing, etc.

8. Advanced Layouts:
   - ui.table(columns, |ui| { ... })
   - ui.tree_node(text, |ui| { ... }) -> bool (open)
   - ui.collapsing_header(text, |ui| { ... }) -> bool (open)
   - ui.scrollable_area(size, |ui| { ... })

9. Window/Panel System:
   - ui.window(title, &mut bool, |ui| { ... })
   - ui.panel(side, |ui| { ... })
   - ui.popup(id, |ui| { ... })
   - ui.tooltip(|ui| { ... })

This design follows the imgui immediate-mode philosophy where:
- State is managed externally
- Functions return interaction results
- Layout is declarative but flexible
- Common patterns are abstracted into helper functions
- Everything is composable
*/

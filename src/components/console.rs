use dioxus::prelude::*;

#[component]
pub fn Console() -> Element {
    rsx! {
        main { class: "flex-1 min-h-0 mx-4 mb-0 mt-0 relative group/console",
            div { class: "absolute inset-0 bg-console-bg rounded-t-2xl border-t border-x border-[#222629] shadow-[inset_0_0_20px_rgba(0,0,0,0.8)] overflow-hidden flex flex-col",
                div { class: "absolute inset-0 scanlines opacity-20 pointer-events-none z-10" }
                div { class: "shrink-0 h-6 bg-[#16181a] border-b border-[#222629] flex items-center justify-between px-3",
                    div { class: "flex gap-1.5",
                        div { class: "w-2 h-2 rounded-full bg-[#394f56]" }
                        div { class: "w-2 h-2 rounded-full bg-[#394f56]" }
                        div { class: "w-2 h-2 rounded-full bg-[#394f56]" }
                    }
                    div { class: "flex items-center gap-2",
                        div { class: "scroll-status-active hidden text-[9px] font-mono text-primary/60 uppercase tracking-widest", "Tracking Bottom" }
                        div { class: "scroll-status-paused hidden text-[9px] font-mono text-yellow-500/60 uppercase tracking-widest", "Scroll Paused" }
                        div { class: "text-[9px] font-mono text-[#4a555a] uppercase tracking-widest", "/dev/tty.usbserial" }
                    }
                }
                div {
                    class: "flex-1 overflow-y-auto p-4 font-mono text-xs md:text-sm leading-relaxed space-y-0.5 scrollbar-custom",
                    id: "console-output",
                    div { class: "flex gap-2 opacity-50",
                        span { class: "log-timestamp text-[#4a555a] shrink-0 tabular-nums select-none", "[10:41:58]" }
                        span { class: "text-gray-500", "Connecting to port COM3..." }
                    }
                    div { class: "flex gap-2",
                        span { class: "log-timestamp text-[#4a555a] shrink-0 tabular-nums select-none", "[10:42:01]" }
                        span { class: "text-gray-300", "System initialization started" }
                    }
                    div { class: "flex gap-2",
                        span { class: "log-timestamp text-[#4a555a] shrink-0 tabular-nums select-none", "[10:42:01]" }
                        span { class: "text-gray-300", "Bootloader v2.1.4 check... " span { class: "text-gray-300", "PASS" } }
                    }
                    div { class: "flex gap-2",
                        span { class: "log-timestamp text-[#4a555a] shrink-0 tabular-nums select-none", "[10:42:02]" }
                        span { class: "text-gray-300", "Loading kernel modules" }
                    }
                    div { class: "flex gap-2",
                        span { class: "log-timestamp text-[#4a555a] shrink-0 tabular-nums select-none", "[10:42:02]" }
                        span { class: "text-gray-300", "Sensor array: " }
                        span { class: "text-gray-300", "CALIBRATING" }
                    }
                    div { class: "flex gap-2 bg-white/5 -mx-4 px-4 py-0.5 border-l-2 border-primary",
                        span { class: "log-timestamp text-[#4a555a] shrink-0 tabular-nums select-none", "[10:42:03]" }
                        span { class: "text-gray-300", span { class: "text-blue-400 font-bold", "RX" } ": <DATA_PACKET_01 id=\"442\" val=\"0x4F\">" }
                    }
                    div { class: "flex gap-2",
                        span { class: "log-timestamp text-[#4a555a] shrink-0 tabular-nums select-none", "[10:42:03]" }
                        span { class: "text-gray-300", "Sensor read: OK" }
                    }
                    div { class: "flex gap-2",
                        span { class: "log-timestamp text-[#4a555a] shrink-0 tabular-nums select-none", "[10:42:04]" }
                        span { class: "text-gray-300", "Telemetry stream active" }
                    }
                    div { class: "flex gap-2",
                        span { class: "log-timestamp text-[#4a555a] shrink-0 tabular-nums select-none", "[10:42:05]" }
                        span { class: "text-gray-400 font-light", "T: 24.5C  H: 45%  P: 1013hPa" }
                    }
                    div { class: "flex gap-2",
                        span { class: "log-timestamp text-[#4a555a] shrink-0 tabular-nums select-none", "[10:42:06]" }
                        span { class: "text-gray-400 font-light", "T: 24.6C  H: 45%  P: 1013hPa" }
                    }
                    div { class: "flex gap-2",
                        span { class: "log-timestamp text-[#4a555a] shrink-0 tabular-nums select-none", "[10:42:07]" }
                        span { class: "text-gray-400 font-light", "T: 24.5C  H: 46%  P: 1012hPa" }
                    }
                    div { class: "flex gap-2",
                        span { class: "log-timestamp text-[#4a555a] shrink-0 tabular-nums select-none", "[10:42:08]" }
                        span { class: "text-gray-300", span { class: "bg-red-500/20 text-red-400 font-bold px-1 -ml-1 rounded", "Warning" } ": Pressure drift detected (-1hPa)" }
                    }
                    div { class: "flex gap-2",
                        span { class: "log-timestamp text-[#4a555a] shrink-0 tabular-nums select-none", "[10:42:09]" }
                        span { class: "text-gray-400 font-light", "T: 24.7C  H: 44%  P: 1012hPa" }
                    }
                    div { class: "flex gap-2",
                        span { class: "log-timestamp text-[#4a555a] shrink-0 tabular-nums select-none", "[10:42:10]" }
                        span { class: "text-gray-400 font-light", "T: 24.8C  H: 44%  P: 1012hPa" }
                    }
                    div { class: "flex gap-2",
                        span { class: "log-timestamp text-[#4a555a] shrink-0 tabular-nums select-none", "[10:42:11]" }
                        span { class: "text-gray-400 font-light", "T: 24.8C  H: 43%  P: 1011hPa" }
                    }
                    div { class: "flex gap-2",
                        span { class: "log-timestamp text-[#4a555a] shrink-0 tabular-nums select-none", "[10:42:12]" }
                        span { class: "text-gray-300", "Heartbeat signal received..." }
                    }
                }
                label {
                    class: "absolute bottom-6 right-6 bg-primary text-[#121416] rounded-full w-10 h-10 shadow-lg shadow-black/50 hover:bg-white active:scale-95 transition-all duration-300 z-20 flex items-center justify-center cursor-pointer group/fab",
                    "for": "autoscroll-toggle",
                    id: "go-to-bottom-btn",
                    span { class: "material-symbols-outlined text-[20px] font-bold", "arrow_downward" }
                    span { class: "absolute -top-8 right-0 bg-surface text-[9px] font-bold text-gray-300 px-2 py-1 rounded border border-white/5 opacity-0 group-hover/fab:opacity-100 transition-opacity whitespace-nowrap pointer-events-none uppercase tracking-widest",
                        "Resume Scroll"
                    }
                }
            }
        }
    }
}

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy},
    window::WindowBuilder,
};
use wry::{WebView, WebViewBuilder};

#[derive(Debug)]
enum UserEvent {
    Navigate(String),
    Back,
    Forward,
    Reload,
}

fn clean_url(input: &str) -> String {
    let value = input.trim();
    if value.is_empty() {
        return "https://www.google.com".to_string();
    }
    if value.starts_with("http://") || value.starts_with("https://") {
        value.to_string()
    } else if value.contains(' ') || !value.contains('.') {
        format!("https://www.google.com/search?q={}", value.replace(' ', "+"))
    } else {
        format!("https://{value}")
    }
}

fn main() -> wry::Result<()> {
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy: EventLoopProxy<UserEvent> = event_loop.create_proxy();

    let window = WindowBuilder::new()
        .with_title("Sleepy Browser")
        .with_inner_size(tao::dpi::LogicalSize::new(1280.0, 800.0))
        .with_min_inner_size(tao::dpi::LogicalSize::new(760.0, 520.0))
        .build(&event_loop)
        .expect("Не удалось создать окно");

    let ipc_proxy = proxy.clone();
    let webview = WebViewBuilder::new()
        .with_html(START_PAGE)
        .with_initialization_script(OVERLAY_SCRIPT)
        .with_ipc_handler(move |request| {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(request.body()) {
                let action = value.get("action").and_then(|v| v.as_str()).unwrap_or("");
                match action {
                    "navigate" => {
                        if let Some(url) = value.get("url").and_then(|v| v.as_str()) {
                            let _ = ipc_proxy.send_event(UserEvent::Navigate(clean_url(url)));
                        }
                    }
                    "back" => { let _ = ipc_proxy.send_event(UserEvent::Back); }
                    "forward" => { let _ = ipc_proxy.send_event(UserEvent::Forward); }
                    "reload" => { let _ = ipc_proxy.send_event(UserEvent::Reload); }
                    _ => {}
                }
            }
        })
        .build(&window)?;

    let mut _webview: WebView = webview;
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::UserEvent(UserEvent::Navigate(url)) => {
                let _ = _webview.load_url(&url);
            }
            Event::UserEvent(UserEvent::Back) => {
                let _ = _webview.evaluate_script("history.back();");
            }
            Event::UserEvent(UserEvent::Forward) => {
                let _ = _webview.evaluate_script("history.forward();");
            }
            Event::UserEvent(UserEvent::Reload) => {
                let _ = _webview.evaluate_script("location.reload();");
            }
            _ => {}
        }
    });
}

const START_PAGE: &str = r##"<!doctype html>
<html lang="ru">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Sleepy Browser</title>
<style>
*{box-sizing:border-box}html,body{height:100%;margin:0}
body{font-family:"Segoe UI",Arial,sans-serif;color:#fff;overflow:hidden;background:
radial-gradient(ellipse at 18% 53%,rgba(215,215,215,.48),transparent 27%),
radial-gradient(ellipse at 78% 83%,rgba(0,0,0,.9),transparent 38%),
radial-gradient(ellipse at 49% 8%,#aaa 0%,#9b9b9b 42%,#262626 100%);}
body:before{content:"";position:absolute;inset:-40px;background:
linear-gradient(155deg,transparent 42%,rgba(20,20,20,.3) 55%,rgba(0,0,0,.82) 88%);
filter:blur(28px);pointer-events:none}
main{position:relative;height:100%;display:flex;align-items:center;flex-direction:column}
.clock{margin-top:105px;font-size:88px;font-weight:300;letter-spacing:-3px;line-height:1}
.date{font-size:16px;margin-top:9px;color:rgba(255,255,255,.9)}
.weather{position:absolute;top:38%;display:flex;align-items:center;gap:10px;font-size:24px}
.sun{color:#ffd52a;font-size:34px}
.search{position:absolute;top:56.7%;width:min(350px,70vw);height:42px;border-radius:12px;
background:rgba(255,255,255,.22);border:1px solid rgba(255,255,255,.13);
box-shadow:0 5px 14px rgba(0,0,0,.13);backdrop-filter:blur(14px);
display:flex;align-items:center;padding:0 15px;transition:.2s}
.search:focus-within{background:rgba(255,255,255,.3);border-color:rgba(255,255,255,.35)}
.search input{width:100%;border:0;outline:0;background:transparent;color:white;font:15px "Segoe UI",sans-serif;text-align:center}
.search input::placeholder{color:rgba(255,255,255,.0)}
.settings{position:absolute;right:28px;bottom:25px;font-size:32px;cursor:pointer;opacity:.95}
.hint{position:absolute;bottom:26px;left:28px;font-size:12px;color:rgba(255,255,255,.45)}
</style>
</head>
<body>
<main>
  <div class="clock" id="clock">15:00</div>
  <div class="date" id="date">Пятница, 20 ноября</div>
  <div class="weather"><span class="sun">☼</span><span>14°</span></div>
  <form class="search" id="search"><input id="q" autocomplete="off" placeholder="Поиск в интернете" aria-label="Поиск или адрес"></form>
  <div class="hint">Sleepy Browser · спокойного сёрфинга</div>
  <div class="settings" title="Настройки" onclick="alert('Настройки Sleepy Browser появятся в следующих версиях.')">⚙</div>
</main>
<script>
function tick(){
 const d=new Date();
 document.getElementById('clock').textContent=d.toLocaleTimeString('ru-RU',{hour:'2-digit',minute:'2-digit',hour12:false});
 document.getElementById('date').textContent=d.toLocaleDateString('ru-RU',{weekday:'long',day:'numeric',month:'long'});
}
tick();setInterval(tick,1000);
function send(data){window.ipc.postMessage(JSON.stringify(data))}
document.getElementById('search').addEventListener('submit',e=>{
 e.preventDefault();const v=document.getElementById('q').value.trim();if(v)send({action:'navigate',url:v});
});
</script>
</body></html>"##;

const OVERLAY_SCRIPT: &str = r##"
(() => {
  if (window.__sleepyOverlayInstalled) return;
  window.__sleepyOverlayInstalled = true;
  const add = () => {
    if (document.title === 'Sleepy Browser') return;
    if (!document.body || document.getElementById('__sleepy_bar')) return;
    const bar = document.createElement('div');
    bar.id = '__sleepy_bar';
    bar.innerHTML = `
      <button title="Назад" data-a="back">‹</button>
      <button title="Вперёд" data-a="forward">›</button>
      <button title="Обновить" data-a="reload">↻</button>
      <form id="__sleepy_form"><input id="__sleepy_url" spellcheck="false"></form>
      <button title="На главную" data-a="home">⌂</button>`;
    const style = document.createElement('style');
    style.textContent = `
      #__sleepy_bar{position:fixed;z-index:2147483647;top:8px;left:8px;right:8px;height:44px;
        display:flex;align-items:center;gap:6px;padding:5px 9px;border-radius:13px;
        background:rgba(30,30,30,.84);backdrop-filter:blur(18px);border:1px solid rgba(255,255,255,.13);
        box-shadow:0 4px 18px #0003;font:14px "Segoe UI",sans-serif}
      #__sleepy_bar button{width:32px;height:30px;border:0;border-radius:8px;background:#ffffff17;color:#fff;font-size:21px;cursor:pointer}
      #__sleepy_bar button:hover{background:#ffffff30}
      #__sleepy_form{flex:1;min-width:0}
      #__sleepy_url{width:100%;height:30px;border:0;border-radius:8px;outline:0;padding:0 12px;
        background:#ffffff16;color:white;font:13px "Segoe UI",sans-serif}
    `;
    document.documentElement.appendChild(style);
    document.body.appendChild(bar);
    const url = bar.querySelector('#__sleepy_url');
    url.value = location.href;
    bar.querySelector('#__sleepy_form').addEventListener('submit', e => {
      e.preventDefault(); window.ipc.postMessage(JSON.stringify({action:'navigate',url:url.value}));
    });
    bar.querySelectorAll('button').forEach(b => b.addEventListener('click', () => {
      const a=b.dataset.a;
      if(a==='home') window.ipc.postMessage(JSON.stringify({action:'navigate',url:'https://www.google.com'}));
      else window.ipc.postMessage(JSON.stringify({action:a}));
    }));
  };
  if(document.readyState==='loading')document.addEventListener('DOMContentLoaded',add,{once:true});else add();
})();
"##;

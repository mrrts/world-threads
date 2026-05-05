use app_lib::ai::openai::{
    audit_injection_state_chat, audit_injection_state_chat_stream,
    audit_injection_state_chat_stream_silent, audit_injection_state_vision, ChatMessage,
    VisionContent, VisionMessage,
};

fn main() {

    for enabled in [false, true] {
        unsafe {
            std::env::set_var(
                "WORLDTHREADS_CHILDREN_MODE",
                if enabled { "1" } else { "0" },
            );
        }
        println!(
            "=== injection audit pass: children_mode={} ===",
            if enabled { "on" } else { "off" }
        );

        let mut chat_messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "ping".to_string(),
        }];
        let (m, c, r) = audit_injection_state_chat(&mut chat_messages);
        println!("chat_audit mission={m} custodiem={c} ryan={r}");

        let mut stream_messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "ping stream".to_string(),
        }];
        let (m, c, r) = audit_injection_state_chat_stream(&mut stream_messages);
        println!("chat_stream_audit mission={m} custodiem={c} ryan={r}");

        let mut stream_silent_messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "ping stream silent".to_string(),
        }];
        let (m, c, r) = audit_injection_state_chat_stream_silent(&mut stream_silent_messages);
        println!("chat_stream_silent_audit mission={m} custodiem={c} ryan={r}");

        let mut vision_messages = vec![VisionMessage {
                role: "user".to_string(),
                content: vec![VisionContent {
                    content_type: "text".to_string(),
                    text: Some("Describe this image.".to_string()),
                    image_url: None,
                }],
        }];
        let (m, c, r) = audit_injection_state_vision(&mut vision_messages);
        println!("vision_audit mission={m} custodiem={c} ryan={r}");
    }
}

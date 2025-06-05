@@ -150,26 +150,25 @@ mod tests {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_raw("{\"choices\":[{\"text\":\"gif-url\"}]}", "application/json"),
            )
            .mount(&server)
            .await;

        let engine = Engine::default();
        let plugin = Plugin::load(&engine, "../../plugins/gif_search.wasm").unwrap();
        let cfg = LlmConfig {
            provider: llm_client::Provider::Ollama,
            base_url: server.uri(),
            api_key: None,
            model: "test".into(),
        };
        let mut inst = plugin
            .instantiate(&engine, cfg, "cats".into(), "cats".into())
            .unwrap();
        inst.invoke("run").unwrap();
        assert_eq!(inst.output(), "gif-url");
    }
}

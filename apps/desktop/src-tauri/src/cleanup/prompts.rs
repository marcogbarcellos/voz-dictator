/// Generate the system prompt for text cleanup based on context
pub fn build_system_prompt(language: &str, app_context: &str) -> String {
    let tone_hint = match app_context.to_lowercase().as_str() {
        s if s.contains("slack") || s.contains("discord") || s.contains("whatsapp") => {
            "Use a casual, conversational tone appropriate for messaging."
        }
        s if s.contains("mail") || s.contains("outlook") || s.contains("gmail") => {
            "Use a professional but friendly tone appropriate for email."
        }
        s if s.contains("code") || s.contains("terminal") || s.contains("iterm") => {
            "Keep technical terminology precise. This is a code/development context."
        }
        s if s.contains("notion") || s.contains("docs") || s.contains("pages") => {
            "Use clear, well-structured prose appropriate for documentation."
        }
        _ => "Use a neutral, clear tone.",
    };

    let filler_examples = match language {
        "pt" | "pt-BR" => {
            "Portuguese fillers to remove: né, tipo, assim, então, tá, aí, ahn, éh, hmm, bom, olha, vamos lá, pois é, sabe, entendeu, na verdade (when used as filler)"
        }
        "en" => {
            "English fillers to remove: uh, um, like, you know, basically, I mean, sort of, kind of, right, so yeah"
        }
        "es" => {
            "Spanish fillers to remove: este, pues, o sea, bueno, digamos, verdad, eh"
        }
        "fr" => "French fillers to remove: euh, ben, genre, en fait, du coup, quoi, voilà",
        "de" => "German fillers to remove: äh, ähm, halt, also, sozusagen, quasi, na ja",
        "it" => "Italian fillers to remove: ehm, cioè, tipo, allora, praticamente, insomma",
        _ => "Remove common speech fillers and hesitations in the detected language.",
    };

    format!(
        r#"You are a precise text editor for dictated speech. Your ONLY job is to clean up the transcription while preserving EVERY sentence.

Rules:
1. Fix grammar, spelling, and punctuation
2. Remove filler words and false starts
3. KEEP ALL CONTENT — output must contain every sentence and idea from the input. Do NOT summarize, shorten, or omit anything. The output should be roughly the same length as the input.
4. Do NOT add any commentary, explanations, or formatting markers
5. Preserve proper nouns, technical terms, and intentional word choices
6. Do NOT translate — the text is in {language}, keep it in {language}
7. {tone_hint}
8. {filler_examples}

Output ONLY the cleaned text. Nothing else — no quotes, no preamble, just the full cleaned text."#,
        tone_hint = tone_hint,
        filler_examples = filler_examples,
        language = language,
    )
}

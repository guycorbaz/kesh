//! Les prompts du serveur MCP — le socle (AC-a24, C-32-6-23).
//!
//! **Aucune implémentation** en 32-6 : `prompts/list` rend une liste vide et
//! `prompts/get` refuse tout nom (`-32602`). Le trait est **scellé** — seul ce
//! module peut l'implémenter (garde `prompts_scelles`) — et `rendre` ne reçoit
//! aucune `Lecture` : un prompt ne porte pas de donnée lue. La 32-10a amendera
//! ce trait en connaissance de cause.

mod scelle {
    /// Le sceau : privé à `mcp::prompts`.
    pub trait Scelle {}
}

/// Un prompt servi par `prompts/get`.
pub trait McpPrompt: scelle::Scelle + Send + Sync + 'static {
    /// Nom unique.
    fn nom(&self) -> &'static str;
    /// Description servie à `prompts/list`.
    fn description(&self) -> &'static str;
    /// Le texte du prompt — un texte du binaire, sans lecture des livres.
    fn rendre(&self) -> &'static str;
}

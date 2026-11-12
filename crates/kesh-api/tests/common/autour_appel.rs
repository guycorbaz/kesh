//! Ce qu'une fonction fait **autour d'un appel** : la garde de structure « rien
//! ne s'attend après l'écriture, ni après le commit », lue par `syn`.
//!
//! Écrite à la Story 33-6 (#450) dans `invoice_settlement_response_e2e.rs`, mise
//! en commun par la Story 34-8 (#938, C-e343-11) pour servir aussi la garde des
//! factures fournisseur (`supplier_invoice_response_e2e.rs`). Le visiteur et ses
//! limites passent **sans changement** ; s'y ajoute [`fn_calls`], qui dit si le
//! corps d'une fonction appelle un nom donné.
//!
//! Inclus par `#[path = "common/autour_appel.rs"] mod autour_appel;` — **pas** par
//! `mod common;`, qui tirerait l'amorce de `common/mod.rs` dans des binaires qui
//! ne l'emploient pas (patron de `common/manuel.rs`).
//!
//! ⚠️ Les sources se lisent par `include_str!` **dans le fichier de test**, placé à
//! `tests/`, qui les passe à ces fonctions : un `include_str!` écrit ici se
//! résoudrait depuis `tests/common/`.

#![allow(dead_code)] // chaque binaire de test n'emploie qu'une partie du module

use syn::visit::{self, Visit};

/// Ce qu'une fonction fait autour d'un appel : les `.await` de tout son corps qui
/// suivent celui de l'appel, et si `read` l'a précédé.
///
/// **Le critère** (revue P2 de la 33-6, P2-1) : l'`.await` de l'appel est le
/// dernier du corps, hors `rollback`, où qu'il soit : instruction suivante, bras
/// d'un `match` ou d'un `if let` sur l'appel, même instruction. Ce n'est plus le
/// découpage en blocs qui décide. Un commit dans un bras, suivi d'un autre bras
/// sans lui, compte donc ce second bras : la garde rougit plutôt que de se taire.
///
/// ⚠️ **Ce qui lui reste invisible, et qu'elle ne prétend pas voir** : une attente
/// dans les jetons d'une macro (`tokio::join!`, `try_join!` : `syn` ne les lit pas) ;
/// une relecture dans une fonction appelée, dont le corps n'est pas lu ; une tâche
/// lancée à part (`tokio::spawn`) sans `.await`. La revue reste le filet de ces
/// formes, sans qu'on en ajoute une liste ici.
///
/// ⚠️ **Et ses écarts propres** (revue P3 de la 33-6, P3-2) : l'ordre suivi est
/// l'ordre **textuel** de `syn`, qui n'est celui de l'évaluation qu'hors boucles et
/// blocs différés ; une boucle qui rejoue après le commit une attente écrite avant
/// lui échappe donc, sans bruit. Les attentes logées dans la **base** de l'`.await`
/// de l'appel (`appel().and_then(|_| relire()).await`) sont vues avant lui, et ne
/// comptent pas. Est exempté tout `.await` dont la base **contient** un `rollback`,
/// et non le seul `tx.rollback().await` : sans portée sur la propriété gardée, le
/// commit n'ayant pas eu lieu sur ces chemins. Un bloc `async` défini avant d'être
/// attendu, ou une branche morte qui porte l'appel, font au contraire rougir à tort.
///
/// ⛔ **Un ensemble clos, non une liste de formes** (revue P1 de la 33-6, B2, B3,
/// E-5) : une liste de relectures interdites laissait passer
/// `factures::detail(.., &state.lecture(), id)` et sautait sur
/// `.await.map_err(..)?;`. Lue par `syn`, la règle ne dépend ni de la forme de la
/// relecture, ni de celle de la fin de l'appel. Une lecture dans la transaction ne
/// se distingue pas d'une lecture après le commit par son résultat (C-e334-10),
/// d'où cette preuve de structure.
#[derive(Debug, PartialEq)]
pub struct AroundCall {
    pub awaits_after: usize,
    pub read_before: bool,
}

/// Le nom de l'appel d'un `Expr`, méthode ou fonction, s'il en porte un.
struct Contains<'a> {
    name: &'a str,
    found: bool,
}

impl<'ast> Visit<'ast> for Contains<'_> {
    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        if let syn::Expr::Path(path) = &*call.func {
            self.found |= path
                .path
                .segments
                .last()
                .is_some_and(|s| s.ident == self.name);
        }
        visit::visit_expr_call(self, call);
    }
    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        self.found |= call.method == self.name;
        visit::visit_expr_method_call(self, call);
    }
}

/// Vrai si `expr` appelle `name`, en fonction (dernier segment du chemin) ou en
/// méthode, à quelque profondeur que ce soit (fermetures comprises).
pub fn expr_calls(expr: &syn::Expr, name: &str) -> bool {
    let mut visitor = Contains { name, found: false };
    visitor.visit_expr(expr);
    visitor.found
}

/// Parcourt les `.await` du corps dans leur ordre textuel, qui est celui de
/// l'évaluation hors boucles et blocs différés (voir les écarts ci-dessus) :
/// l'intérieur d'un `.await` avant lui, le scrutateur d'un `match` avant ses bras.
/// Le premier qui porte l'appel est le sien ; tout `.await` suivant compte, sauf
/// un `rollback`.
struct Around<'a> {
    target: &'a str,
    read: Option<&'a str>,
    found: bool,
    awaits_after: usize,
    read_before: bool,
}

impl<'ast> Visit<'ast> for Around<'_> {
    fn visit_expr_await(&mut self, node: &'ast syn::ExprAwait) {
        visit::visit_expr_await(self, node);
        if self.found {
            if !expr_calls(&node.base, "rollback") {
                self.awaits_after += 1;
            }
        } else if expr_calls(&node.base, self.target) {
            self.found = true;
        } else if let Some(read) = self.read {
            self.read_before |= expr_calls(&node.base, read);
        }
    }
}

/// La fonction libre `function` de `source` (un fichier Rust entier).
fn find_fn<'a>(file: &'a syn::File, function: &str) -> &'a syn::ItemFn {
    file.items
        .iter()
        .find_map(|item| match item {
            syn::Item::Fn(f) if f.sig.ident == function => Some(f),
            _ => None,
        })
        .unwrap_or_else(|| panic!("fonction introuvable : {function}"))
}

/// Ce que `function` de `source` fait autour de son appel à `target` (voir
/// [`AroundCall`]). Panique si la fonction ou l'appel sont introuvables : une
/// garde qui ne trouve pas sa cible ne doit pas se taire.
pub fn around(source: &str, function: &str, target: &str, read: Option<&str>) -> AroundCall {
    let file = syn::parse_file(source).expect("source lisible");
    let item = find_fn(&file, function);
    let mut visitor = Around {
        target,
        read,
        found: false,
        awaits_after: 0,
        read_before: false,
    };
    visitor.visit_block(&item.block);
    assert!(visitor.found, "{function} : appel de {target} introuvable");
    AroundCall {
        awaits_after: visitor.awaits_after,
        read_before: visitor.read_before,
    }
}

/// Vrai si le corps de la fonction libre `function` de `source` appelle `name`
/// (Story 34-8, AC-3) : sert à dire qu'une enveloppe termine sa transaction par
/// `commit_with_view` et **non** par `commit`. Le nom se compare **en entier**
/// (`commit` ne reconnaît pas `commit_with_view`). Mêmes angles morts que
/// [`around`] : les jetons d'une macro ne sont pas lus.
pub fn fn_calls(source: &str, function: &str, name: &str) -> bool {
    let file = syn::parse_file(source).expect("source lisible");
    let item = find_fn(&file, function);
    let mut visitor = Contains { name, found: false };
    visitor.visit_block(&item.block);
    visitor.found
}

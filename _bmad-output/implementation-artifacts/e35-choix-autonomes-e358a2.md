# Epic 35 — choix autonomes de la story 35-8a2 (clé `e358a2`, #431)

Registre tenu par l'agent de développement. Mise à jour par union, jamais par écrasement.

## Base de la branche

### C-e358a2-1 — Fusion de la 35-3 : le côté de la base est gardé dans les onze fichiers de texte

- **Contexte** : la fusion de `story/35-3-comptes-designes-type` dans la base (35-5c, 35-5b, 35-8a1 avec la 35-1) rend
  onze conflits de texte (`CHANGELOG.md`, `sprint-status.yaml`, les deux `errors.rs`, `textes_coherents.rs`, les quatre
  `messages.ftl`, `docs/api-external.md`, `invoices/[id]/+page.svelte`). Dans chacun, le bloc du côté 35-3 est **vide** :
  la variante `DesignatedAccountsWrongType`, ses clés, ses lignes de garde, sa ligne d'API et son entrée de CHANGELOG
  sont déjà dans la base (portées par la 35-1, partie de la 35-3), et le bloc de la base ajoute ce que la 35-1 pose
  ensuite (`NUMBER_FORMAT_UNRENDERABLE`).
- **Retenu** : l'union, qui se réduit ici au côté de la base, marqueurs retirés par l'outil Edit. Comparé au même fichier
  de l'intégration à blanc (`e2e1c39c4`) : les deux `errors.rs`, `textes_coherents.rs` et la page Svelte sont identiques ;
  les écarts des `messages.ftl`, d'`api-external.md` et du CHANGELOG viennent des stories absentes ici (35-4, solde
  d'ouverture #1025-#1028, #1000) et de la 35-5c, présente ici et non dans l'intégration à blanc.
- **Écartées** : prendre le côté 35-3 (perdrait la 35-1) ; réécrire à la main.
- **Réversible** : oui (commit de fusion `4531669f8`).

### C-e358a2-2 — `sprint-status.yaml` : 35-2a et 35-2b gardées à `done`

- **Contexte** : seul conflit qui ne soit pas une union — la même clé, `done` dans la base, `ready-for-dev` du côté 35-3
  (relevé antérieur à leur développement).
- **Retenu** : `done`, l'état le plus récent, comme l'intégration à blanc.
- **Réversible** : oui.

### C-e358a2-3 — PDF des manuels : côté de la base, puis régénérés

- **Contexte** : `admin-manual.pdf` et `user-manual.pdf` en conflit binaire.
- **Retenu** : `git checkout --ours`, puis `make -B user admin` dans `docs/manual/` sous `mem-guard`
  (`kesh-gate-logs/e358a2/base-pdf.log`) : `-B` parce que le PDF repris est plus récent que les `.tex` et que `make`
  l'aurait cru à jour.
- **Réversible** : oui.

## Développement

### C-e358a2-4 — Le règlement fournisseur passé par un rapprochement porte la clé (`pay_in_tx_as`)

- **Contexte** : `supplier_invoices::pay_in_tx_as` reçoit déjà un `letterings::Actor` (= `Acteur`) : la clé du
  rapprochement depuis `accept_one_supplier_invoice` (`routes/reconciliation.rs`), un acteur sans clé depuis `pay_in_tx`
  (la route `POST /supplier-invoices/{id}/pay`). Son `create_in_tx` recevait `actor.user_id`. La fiche ne nomme pas ce
  site parmi les « six qui ont la clé », mais AC6b dit que les écritures « qu'un rapprochement bancaire passe » sont
  attribuées à la clé.
- **Retenu** : `create_in_tx(…, actor, …)` — l'acteur entier. Le paiement par la route reste sans clé (le site
  `ApiKeyNone` de `pay_in_tx`, `AConvertir`, inchangé) ; le paiement par rapprochement porte la clé, comme le manuel le dit.
- **Écartées** : `Acteur::utilisateur(user_id)` (un site `AConvertir` de plus, et un écart entre le manuel et le code
  pour le rapprochement d'une facture fournisseur).
- **Réversible** : oui.

### C-e358a2-5 — `create_opening_entry` garde `user_id: i64`

- **Contexte** : le bilan d'ouverture (`POST /opening-balances`, `routes/opening_balances.rs`) appelle
  `journal_entries::create_opening_entry(…, current_user.user_id, …)`, qui appelle `create_in_tx`. La fiche ne la nomme
  pas ; AC6b classe « un solde d'ouverture » parmi ce qui reste au nom du créateur de la clé.
- **Retenu** : la signature ne change pas ; son appel interne passe `Acteur::utilisateur(user_id)`, site `AConvertir` de
  la garde (`journal_entries.rs`, `create_opening_entry`), au déploiement (Epic 36).
- **Écartées** : la convertir à l'acteur (contredirait AC6b et élargirait la story).
- **Réversible** : oui.

### C-e358a2-6 — Conversion des tests confiée à trois agents, fichiers disjoints

- **Contexte** : ≈ 137 appels de tests dans 48 fichiers (relevé du compilateur, `kesh-gate-logs/e358a2/conversion-erreurs-2.txt`).
- **Retenu** : trois agents Sonnet, un lot de fichiers chacun (`lot-00` à `lot-02`), outil Edit seul, sans compilation ;
  le compilateur, puis `cargo fmt`, contrôlent le résultat. Les appels du `mod tests` de `journal_entries.rs` ont été
  convertis par moi (Edit `replace_all` sur des motifs vérifiés).
- **Réversible** : oui.

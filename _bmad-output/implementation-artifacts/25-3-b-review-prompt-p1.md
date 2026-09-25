# Prompt — passe 1 de `bmad-code-review`, Story 25-3-b

*Versionné le 2026-09-25. Trois lentilles en contexte frais (Sonnet), orthogonales à l'auteur du
code (Opus). Protocole du workflow : Blind Hunter, Edge Case Hunter, Acceptance Auditor.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-3-b-annuler-rapprochement`, tête `80bd4e72`.
**Diff revu** : `git diff main..HEAD` sur le code et la documentation (PDF et fiche exclus),
enregistré dans
`/tmp/claude-1000/-home-gcorbaz-devel-kesh/5bef2124-6ec8-41d8-9004-39b0653420a3/scratchpad/25-3-b-code.diff`
(3924 lignes). Fiche : `_bmad-output/implementation-artifacts/25-3-b-annuler-rapprochement.md` —
ses AC, ses arbitrages (Q1-Q3, Q5 : **ne pas les contester**, en contester la mise en œuvre) et
son Dev Agent Record, qui déclare **trois écarts assumés** à la fiche.

Objet : **annuler un rapprochement bancaire** (#418). Le lien `matched_entry_id` est défait AVANT
la contre-passation ; un règlement client est annulé par le geste de la 25-3-a-1
(`invoice_settlements_write::cancel_settlement_in_tx`), appelé ; une écriture propre (éclatement,
règle, rapprochement manuel) est contre-passée par le socle. Nouveau : le module
`crates/kesh-db/src/repositories/reconciliation_cancel.rs`, l'exemption étroite du rang 3 dans la
queue commune (`settlement_cancellation.rs`), `DbError::ReconciliationNotCancellable`, deux routes
(`routes/reconciliation.rs`, fin du fichier) avec rejeu sur interblocage, le dialogue partagé
`frontend/src/lib/features/reconciliation/CancelReconciliationDialog.svelte`.

⛔ **Le développement a trouvé et corrigé un défaut de concurrence** (Dev Agent Record, Debug Log) :
une lecture non verrouillante faite avant le verrou de l'exercice figeait l'instantané. Le
correctif rend le classement verrouillant et **inverse l'ordre des verrous** prescrit. C'est le
premier suspect.

## Lentille 1 — Blind Hunter (diff SEUL)

Tu ne lis **que** le fichier de diff. Revue adversariale générale : logique, erreurs, concurrence,
multi-tenant (`company_id` sur **chaque** requête), tests qui ne prouvent rien ou passent à vide,
incohérences code / commentaires / textes, clés i18n.

## Lentille 2 — Edge Case Hunter (diff + dépôt)

Chaque branche du code neuf, en lisant l'appelé et l'appelant. Priorités :
1. **L'instantané et les verrous de `cancel_in_tx`** : reste-t-il, AVANT le verrou de l'écriture
   et de l'exercice (étape 3), une lecture non verrouillante — dans `find`, `classify`, ou dans
   ce qu'appellent `cancel_settlement_in_tx` et `reverse_in_tx` ? Après l'étape 3, chaque lecture
   qui décide d'un refus voit-elle l'état juste ? Le rang 1 (facture créditée) est lu par
   `cancel_settlement_in_tx` : son instantané est-il juste face à un avoir concurrent ?
2. **L'ordre des verrous** (transaction bancaire → ligne de règlement → facture → écriture +
   exercice → socle) contre `accept_one_invoice`, `post_manual`, `post_split`, `accept_one_split`,
   `accept_one_rule`, `post_reject`, `cancel_settlement_in_tx` (fiche facture),
   `invoice_settlements_write::settle_invoice`, `credit_notes` (avoir), `fiscal_years::close` /
   `reopen`, `invoices::unvalidate`. Interblocage possible ? Rejoué ou non ?
3. **Le rejeu** (`post_cancel_reconciliation`) : la closure est-elle réellement rejouable, le
   verrou nommé relâché, le prédicat juste, aucun effet de bord hors transaction ?
4. **L'exemption étroite** (`settlement_entry_cancel_blocker(…, Some(id))`) et
   `settlement_cancel_blocker_unlinking` : juste pour chaque `kind` ? Les deux sœurs strictement
   inchangées en comportement ?
5. **Le frontend** : le dialogue (ouverture, relecture, double clic, erreur à la lecture, code hors
   motifs), le détail d'import (rôle, relecture), la fiche facture (le bouton, la relecture).

## Lentille 3 — Acceptance Auditor (diff + fiche + documentation)

Chaque AC (1 à 14) contre le diff ; les **trois écarts** déclarés au Dev Agent Record — sont-ils
justifiés, et écrits partout où il le faut (doc-comments) ? Les décomptes du Dev Agent Record
**recomptés** depuis la source (15 tests Rust, 10 Vitest, 1 Playwright ; registre 109/91/15/3/112 ;
97 actions / 127 libellés ; `sitesTotal` 1709 ; `CANDIDATES_ATTENDUES` 44). Les textes : quatre
locales, replis **mot pour mot** le FTL fr-CH, liste de l'AC 9 **close** (grep de la clé, du code
et de la phrase). Le manuel `docs/manual/fr/user-manual.tex` et son **PDF aplati**
(`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`), `docs/api-external.md`,
`CHANGELOG.md` (section `[0.12.1]` seulement), `README.md`. ⚠️ Cette story **ferme #418** :
reste-t-il dans le dépôt une phrase qui dit qu'un rapprochement ne se défait pas ?

## Ce que tu rends

- Findings : sévérité (CRITICAL / HIGH / MEDIUM / LOW), `fichier:ligne`, **preuve** (code lu,
  commande et résultat), correction. Pour tout CRITICAL ou HIGH affirmant qu'une chose est absente
  ou présente : la commande `grep -nF` exécutée et son résultat.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.**

## Interdits

⛔ N'écris aucun fichier du dépôt, n'exécute aucune commande qui écrit dans le dépôt ou dans une base
persistante — `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make` dans `docs/manual/`, tout `git commit`/`push`/
`add`/`stash`/`reset`/`rebase`/`checkout`, `sqlx migrate`, `cargo test`/`cargo nextest`, `npm run`,
`npx playwright`. Lecture, `grep`, `git log`/`show`/`diff`, `pdftotext` (vers le scratchpad) et
`cargo check` sont autorisés.

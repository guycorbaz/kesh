# Story 35-8a2 : l'audit attribue à la clé d'API les écritures qu'elle passe, contre-passe, modifie ou supprime (#431)

Status: ready-for-dev

*Fiche tirée le 2026-11-13 de la fiche `35-8a-audit-acteur-patron.md` (validation P1 remédiée, même date), par l'agent
de rédaction des fiches de l'Epic 35 (Opus 5.5), worktree `kesh-e35`, sur décision de l'orchestrateur (C-e350-50,
« découpé »). **Aucune réécriture du fond** : c'est le lot 2 de la 35-8a. Story de *rollout* au sens du `CLAUDE.md`
(§ *Règle de splitting préventif*) : la logique est posée par la **35-8a1**, la conversion est mécanique et se revoit
fichier par fichier. Release **0.17.7**. **Aucune migration.** **`kesh-db` touché : gate complet à chaque commit.** Code
lu sur **`main`** (`67efc51f`). Registres : `e35-choix-autonomes-e350.md` (C-e350-8, -38 à -40, -48 à -50) ;
`e35-choix-autonomes-e358a2.md` pour la story, à créer au premier choix. Validation P2 à lancer.*

**Issues** : la PR porte `refs #431` (#431 reste ouverte : le déploiement est reporté à l'Epic 36, C-e350-48). **Aucun
mot-clé de fermeture.**

## Story

En tant que titulaire dont un agent écrit dans Kesh par une **clé d'API REST**, je veux que la piste de contrôle dise,
pour chaque écriture que cet agent passe, contre-passe, modifie ou supprime — à la main ou par un rapprochement bancaire
—, que c'est **la clé** qui l'a fait, afin que le journal d'audit réponde juste à « qui a fait quoi » (CO art. 957-964,
OLICo art. 9).

## Ordre et dépendances

- **Part de `main` après la fusion de la 35-8a1** (le type `Acteur`, `par`, `From<&CurrentUser>`, la garde) **et des
  35-5a0, 35-5a, 35-5b, 35-5c** (`repositories/journal_entries.rs` : la plomberie de l'horloge fait passer la date du jour
  en paramètre dans la même chaîne de fonctions, `create*`, `reverse*` — fiches `35-5a0-horloge-plomberie.md`,
  `35-5c-date-du-jour-tests.md:185`, branche `chore/epic-35-fiches-355`) **et de la 35-3** (ordre de l'epic, § *Ordre et
  dépendances*, point 3). Les numéros de ligne ci-dessous sont ceux de `main` ; l'étape 0 les relit sur la base réelle.

## Ce qui se lit au code (`main`)

**Les fonctions converties** — `journal_entries::create` (`:195`), `create_in_tx` (`:242`), `create_in_tx_inner` (`:281`,
trace `journal_entry.created` `:562`) ; `reverse` (`:3097`), `reverse_in_tx` (`:2558`), `reverse_owned_in_tx` (`:2628`),
`reverse_at_next_fiscal_year_start[_in_tx]` (`:3122`, `:2677`), `reverse_in_tx_inner` (`:2796` ; trace
`journal_entry.reversed` `:3017`, lettrage `reversal` `:3068`, posé seulement sur un compte lettrable `:3054`) ;
`delete_in_tx` (`:1858`, déjà `actor_api_key_id`), `delete_by_id` (`:1756`), `update` (`:1444`, déjà). Routes :
`routes/journal_entries.rs` (`:456` contre-passation, `:489` à l'exercice suivant, `:638` création, `:700` modification,
`:749` suppression).

**Les appelants qui ont déjà la clé en main** (V1-5 de la validation P1 de la 35-8a) : les cinq
`journal_entries::create_in_tx(tx, fiscal_year.id, user_id, new_je, false)` de `kesh-api/src/routes/reconciliation.rs` —
`:2820` (`accept_one_invoice`, `actor_api_key_id` `:2377`), `:3812` (`accept_one_split`, `:3523`), `:4260`
(`accept_one_rule`, `:3951`), `:5130` (`post_manual_once`, `current_user` `:5039`), `:5596` (`post_split_once`,
`current_user` `:5499`) — et `reconciliation_cancel.rs:723` (`reverse_in_tx`). Un rapprochement par clé écrit aujourd'hui
`reconciliation.accepted` à la clé et `journal_entry.created` au créateur de la clé. **Le dé-rapprochement** (`cancel_in_tx`,
`reconciliation_cancel.rs:564-723`, qui a `actor_api_key_id`) appelle, selon le genre, `invoice_settlements_write::
cancel_settlement_in_tx(…, user_id)` (`:705`) et `supplier_invoices::cancel_settlement_in_tx(…, user_id)` (`:718`) — non
convertis par cette story (déploiement, Epic 36) — et `journal_entries::reverse_in_tx(…, user_id)` (`:723`) — converti ici.

**Les autres appelants de production** (`Acteur::utilisateur(user_id)`, sites `A_CONVERTIR` de la garde) :
`credit_notes.rs:697`, `invoice_settlements_write.rs:337`, `:700`, `:1023`, `invoices.rs:1689`, `:2389`,
`opening_complement.rs:719`, `supplier_invoices.rs:1752`, `:2135`, `:2600`, `:2871` ; `kesh-seed/src/pieges.rs:511`
(raison `FIXTURE`). `supplier_invoices.rs:1410`, `:1412` (retour en brouillon) passent déjà l'`acteur` depuis la 35-8a1.

**Le rayon** (relevé sur `main` `67efc51f`, fiche 35-8a, Dev Notes, *Taille*) : production 10 fichiers d'appelants
(≈ 25 appels) plus `journal_entries.rs` ; tests d'intégration 47 fichiers, 129 appels ; `mod tests` de `src`
(`journal_entries.rs` après `:3177`, relevé brut ≈ 83 ; `accounts.rs` 2). **≈ 60 fichiers, ≈ 240 appels**, mécaniques.

## Décisions

- **D3 — Les écritures** (35-8a, D3, reprise telle quelle) : `create`, `create_in_tx`, `create_in_tx_inner` →
  `acteur: Acteur` ; `journal_entry.created` par `NewAuditLogEntry::par` ; `reverse*` → `acteur` ;
  `journal_entry.reversed` et le lettrage `reversal` (`:3068`) portent la clé (l'écart R3 de `letterings.rs:201-202` est
  levé) ; `delete_in_tx`, `delete_by_id`, `update` passent de `(user_id, actor_api_key_id)` à `acteur` (les deux appels du
  retour en brouillon, `supplier_invoices.rs:1410`, `:1412`, passent alors l'`acteur` entier). Les routes de
  `routes/journal_entries.rs` passent `Acteur::from(&current_user)`. Les six appelants qui ont la clé la passent
  (`Acteur { user_id, api_key_id: actor_api_key_id }` ou `Acteur::from(current_user)`) ; les autres passent
  `Acteur::utilisateur(user_id)`, chacun inscrit à la table de la garde (`A_CONVERTIR`, ou `FIXTURE` pour `pieges.rs`), au
  compte exact.
- **D4 — Ce qui ne change pas** : `user_id` = créateur de la clé ; vocabulaire, schéma, affichage, routes et réponses
  inchangés. Les entrées antérieures gardent leur marque (déjà écrit par la 35-8a1).
- **D5 — Hors de la story** : le serveur MCP (#1057) ; la restauration ; le reste des dépôts (35-8b, Epic 36).

## Critères d'acceptation

1. **AC1 — Une écriture passée par clé** (patron `api_keys_e2e.rs:438-505`) : clé `read-write` d'un Comptable,
   `POST /api/v1/journal-entries` → `journal_entry.created` porte `actor_type = 'api_key'`, `actor_api_key_id` = la clé,
   `user_id` = son créateur ; par session → `'user'`, clé nulle. **Et un rapprochement par clé** :
   `POST /api/v1/reconciliation/manual` (`lib.rs:887`) → `journal_entry.created` porte la clé, comme
   `reconciliation.accepted` ; un cas de `POST /api/v1/reconciliation/accept` (`lib.rs:878`) si le montage d'une proposition
   tient dans le test, sinon les trois sites d'`accept_*` couverts par la garde (AC5b) et nommés au Change Log.
2. **AC2 — Une contre-passation par clé** (`POST /api/v1/journal-entries/{id}/reverse`, et
   `…/reverse-at-next-fiscal-year-start`, qui exige un **exercice suivant ouvert** dans le montage) : l'écriture
   contre-passée porte **obligatoirement** une ligne sur le compte **créanciers ou débiteurs** des fixtures — un compte
   lettrable (`letterings.rs:363`, `is_letterable` : `Asset` ou `Liability`, non lié à un compte bancaire — définition
   `BANK_LINKED_SQL`, `:351-356` ; côté documentation, `api-external.md:408`) ; le test asserte d'abord qu'un `lettering.created` d'origine `reversal`
   **existe**, puis que lui et `journal_entry.reversed` portent la clé (sans quoi M3 survivrait sur un montage qui ne lettre
   rien). Modification et suppression par clé (`update`,
   `delete_by_id`) : la trace porte la clé (déjà vrai, à garder vert).
3. **AC4 — Non-régression** : la validation d'une facture client par session reste `'user'` ; un appelant converti en
   `Acteur::utilisateur` écrit exactement l'entrée d'avant (tests de dépôt existants verts) ; l'AC3 de la 35-8a1 reste vert.
   **Limite connue, rendue vérifiable** (validation P2, V2-1) : la validation d'une facture **par clé** écrit encore
   `journal_entry.created` à `'user'` — asserté tel quel, renvoyé à #431 (déploiement, Epic 36).
4. **AC5b — La garde** : table recomptée depuis la source à l'étape finale (les sites des écritures sortis de
   `A_CONVERTIR` ou y restant au compte exact), ventilation et total cohérents au Change Log. **La table ne porte aucune
   entrée `Acteur::utilisateur` pour `routes/reconciliation.rs`, `routes/journal_entries.rs` et
   `repositories/reconciliation_cancel.rs`** : c'est ce qui couvre les cinq sites qui ont la clé dans `reconciliation.rs` et
   le dé-rapprochement (`:723`), que les tests HTTP n'exercent pas tous (V2-2). Le **cinquième motif** de la garde
   (`delete_in_tx(…, None, …)`, 35-8a1) **tombe à zéro** : `delete_in_tx` prend l'`acteur` ; l'entrée de `invoices.rs:1689`
   (`invoices::unvalidate`) passe du motif 5 au motif `Acteur::utilisateur`, toujours `A_CONVERTIR` (validation P3 de la
   35-8a1, V3-4).
5. **AC6b — Manuel et CHANGELOG** (périmètre exact, V2-1) : les paragraphes posés par la 35-8a1 (`admin-manual.tex:3193`,
   `user-manual.tex:3959-3960`) disent : « les écritures saisies, modifiées, supprimées ou contre-passées par les routes du
   journal, et celles qu'un rapprochement bancaire passe, sont attribuées à la clé ; **celles que Kesh passe pour une
   facture (validation, dévalidation), un avoir, un règlement, le solde de son reste ou leur annulation, une facture
   fournisseur (validation, paiement, annulation) ou un solde d'ouverture restent au nom du créateur de la clé** ; un
   dé-rapprochement par clé contre-passe au nom de la clé une écriture rapprochée seule ; celui d'un règlement de facture
   (client ou fournisseur) l'annule, contre-passation comprise, au nom du créateur de la clé ; un simple détachement ne
   contre-passe rien » (renvoi à #431 ; validation P3, V3-1, V3-2 : genre par genre, `reconciliation_cancel.rs:694-725` ;
   `write_off_invoice`, `invoice_settlements_write.rs:700` ; « le créateur de la clé » partout, non « l'utilisateur ») ;
   même phrase au CHANGELOG, *Corrigé*, `refs #431` ; PDF régénérés, contrôlés aplatis.

## Tests à écrire, et la preuve du rouge

AC1 et AC2 écrits d'abord, sur la base de la branche : rouges par l'`actor_type = 'user'`
(`kesh-gate-logs/e358a2/rouges.log`). AC4 vert au départ.

**Mutations** : M2' `create_in_tx_inner` qui trace par `::user` (AC1) ; M3 le lettrage `reversal` rendu à
`api_key_id: None` (AC2, dont le montage lettre obligatoirement — et la garde) ; M7 `post_manual_once` qui passe
`Acteur::utilisateur(user_id)` (AC1) ; **M8** `reconciliation_cancel.rs:723` qui passe `Acteur::utilisateur(user_id)` à
`reverse_in_tx` (AC5b, la garde) ; **M9** un `accept_one_*` de `routes/reconciliation.rs` qui passe
`Acteur::utilisateur(user_id)` (AC5b, la garde ; AC1 si le cas d'`accept` est monté). Toute mutation survivante est écrite au
Change Log avec sa raison.

## Tâches

- [ ] Étape 0 : relire les lignes citées sur la base réelle (après 35-8a1, 35-5a0 à 35-5c, 35-3) ; recompter le rayon et
      l'écrire au Change Log.
- [ ] Rouges d'AC1, AC2 ; signatures des écritures → `acteur` ; les six sites qui ont la clé ; appelants de production ;
      tests convertis (mécanique, fichier par fichier) ; table de la garde recomptée ; manuels ; CHANGELOG.
- [ ] Gates : backend complet à chaque commit (`kesh-db`) ; frontend : rien ne change ; E2E complet au dernier commit de
      code.

## Dev Notes

### Fichiers communs — règle d'union C-e350-8

`repositories/journal_entries.rs` (35-5a0 à 35-5c avant elle) ; les appelants (`invoices.rs` — 35-1 ;
`supplier_invoices.rs`, `invoice_settlements_write.rs`, `credit_notes.rs`, `opening_complement.rs`,
`reconciliation*.rs`) : elle part **après** toutes les autres stories de l'epic qui les touchent ; `CHANGELOG.md` ;
`admin-manual.tex:3193`, `user-manual.tex:3959-3960` (après la 35-8a1) ; la table de la garde ;
`supplier_invoices.rs:1410-1412` (retour en brouillon), **même hunk** que la 35-8a1, réécrit ici après elle (de
`acteur.user_id, acteur.api_key_id` à `acteur`).

### Taille (validation P2, V2-6)

≈ 60 fichiers, ≈ 240 appels, trois natures de travail : les signatures (mécaniques, revue fichier par fichier) ; les six
sites qui ont la clé et le dé-rapprochement ; deux E2E d'API neufs (le rapprochement exige un import bancaire monté). La
revue fichier par fichier ne vaut que pour les signatures ; les six sites et les E2E restent à la boucle complète.

### Ce qui doit rester vrai

- `user_id` reste le créateur de la clé. Aucune entrée d'audit perdue ni doublée. Fixtures synthétiques seulement.

### Références

- Fiche `35-8a-audit-acteur-patron.md` (`split`) : origine ; **en cas d'écart, la 35-8a2 fait foi sur la 35-8a** (validation P3 de la
  35-8a1, V3-5) ; `35-8a1-audit-acteur-mecanisme.md` ;
  `kesh-gate-logs/e35-8a-validation-p1.md`.
- Issues #431 ; Story 17-2a ; Story 15-8 (modification et suppression d'écriture par clé).

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-11-13 — Fiche tirée de la 35-8a (lot 2), sur décision de l'orchestrateur (C-e350-50, « découpé »), sans réécriture
  du fond. P2 à lancer.
- 2026-11-13 — **Validation P2** : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 4 LOW (`kesh-gate-logs/e35-8a2-validation-p2.md`). Remédiée
  par l'agent de rédaction des fiches (Opus 5.5). Prémisses relues sur `main` (`67efc51f`) : `reconciliation_cancel.rs:700-724`
  (`:705`, `:718` passent `user_id`, `:723` `reverse_in_tx`) ; point 3 du fichier d'epic. Confirmées.
  - V2-1 (MEDIUM) : AC6b dit le périmètre exact (ce qui reste au nom de l'utilisateur, dont le dé-rapprochement à moitié),
    même phrase au CHANGELOG ; AC4 rend la limite vérifiable (validation d'une facture par clé → `'user'`).
  - V2-2 (MEDIUM) : le dé-rapprochement décrit ; AC5b écrit que l'absence d'entrée `Acteur::utilisateur` couvre les sites
    de `reconciliation.rs` et `reconciliation_cancel.rs:723` ; mutations M8, M9.
  - LOW : V2-3 (dépendances alignées : point 3 du fichier d'epic précisé ; fiche et `sprint-status.yaml` déjà justes) ; V2-4
    (`supplier_invoices.rs:1410-1412`, même hunk que la 35-8a1) ; V2-5 (AC2 : exercice suivant ouvert, compte créanciers ou
    débiteurs, `lettering.created` asserté d'abord) ; V2-6 (taille et nature de la revue, Dev Notes).
  - Symptômes greppés (fiches 35-8a1, 35-8a2, epic, registre) : « saisie, modification, suppression, contre-passation,
    rapprochement bancaire — à ce qui est attribué » — remplacé ; « 35-8 après » — précisé.
- 2026-11-13 — **Validation P3, passe ciblée** (Opus) sur la remédiation P2 : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 2 LOW
  (`kesh-gate-logs/e35-8a2-validation-p3-ciblee.md`). Remédiée au plus court. Prémisse relue sur `main` :
  `reconciliation_cancel.rs:694-725` (genre `unlink_only` : rien ; règlement client ou fournisseur : `cancel_settlement_in_tx`
  avec `user_id`, contre-passation comprise ; `Entry` : `reverse_in_tx`) ; `letterings.rs:363` (`is_letterable`). Confirmées.
  - V3-1 (MEDIUM) : la phrase du manuel et du CHANGELOG (AC6b) dit le dé-rapprochement genre par genre.
  - LOW : V3-2 (le solde du reste et l'annulation d'un règlement nommés ; « le créateur de la clé » partout) ; V3-3 (AC2 cite
    `letterings.rs:363`, `:351-356`, `api-external.md:408`) ; et, de la P3 de la 35-8a1, V3-4 (le cinquième motif tombe à
    zéro, AC5b) et V3-5 (cette fiche fait foi, *Références*).
  - Symptôme greppé : « annule un règlement au nom de l'utilisateur », « qui fait foi pour le rayon » — remplacés.
- 2026-11-13 — **Validation P4, passe ciblée** (Sonnet) sur la remédiation P3 : 0 finding au-dessus de LOW, 2 LOW
  (`kesh-gate-logs/e35-8a2-validation-p4-ciblee.md`). Appliqués : V4-1 (« au créateur de la clé », l. 46) ; V4-2 (renvoi
  d'autorité sans ambiguïté : « la 35-8a2 fait foi sur la 35-8a », comme la ligne de `sprint-status.yaml`). Rédaction seule.
- 2026-11-13 — **Validation terminée.** Série des findings au-dessus de LOW : P1 (sur la 35-8a entière) 1 HIGH + 7 MEDIUM →
  découpage en 35-8a1 et 35-8a2 (C-e350-50) → P2 2 MEDIUM → P3 1 → P4 0 (CRITICAL : 0). Modèles : Opus (P1), Sonnet (P2),
  Opus en passe ciblée (P3), Sonnet en passe ciblée (P4). La boucle se clôt sur une passe sans finding au-dessus de LOW ; la
  remédiation de P4 ne touche que la rédaction. Choix retenus : C-e350-38 à -40, -48 à -50, -52 (ceux de la 35-8a1, dont la
  garde). Statut : `ready-for-dev`.

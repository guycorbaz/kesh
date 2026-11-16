# Story 35-8a2 : l'audit attribue à la clé d'API les écritures qu'elle passe, contre-passe, modifie ou supprime (#431)

Status: done

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
   *(Annoté le 2026-11-14, remédiation de la revue de code P1, M-A : cette phrase rangeait le règlement d'une facture passé par
   un rapprochement sous deux règles contraires. Les textes livrés la qualifient — « toutes celles qu'un rapprochement bancaire
   passe (y compris le règlement d'une facture client et le paiement d'une facture fournisseur que le rapprochement
   enregistre) », « un règlement saisi sur la facture », « paiement depuis la fiche ou par un lot de paiement ». Le critère
   n'est pas réécrit.)*

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

- [x] Étape 0 : relire les lignes citées sur la base réelle (après 35-8a1, 35-5a0 à 35-5c, 35-3) ; recompter le rayon et
      l'écrire au Change Log.
- [x] Rouges d'AC1, AC2 ; signatures des écritures → `acteur` ; les six sites qui ont la clé ; appelants de production ;
      tests convertis (mécanique, fichier par fichier) ; table de la garde recomptée ; manuels ; CHANGELOG.
- [x] Gates : backend complet au dernier commit de code ; frontend : rien ne change (joué sur la base) ; E2E complet **à la
      clôture** (consigne de l'Epic 35 : pas d'E2E au passage en `review`).

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

Opus 5.5 (Claude Code), worktree `kesh-e358a2`, voie 2 (`kesh-mariadb-dev2`, base `kesh_e358a2`), 2026-11-14. Conversion des
tests confiée à trois agents Sonnet, fichiers disjoints, outil Edit seul (C-e358a2-6).

### Debug Log References

Journaux sous `/home/gcorbaz/devel/kesh-gate-logs/e358a2/` :
- base : `base-pdf.log`, `base-db-reset.log`, `base-test-fast.log`, `base-front-{check,lint-i18n-ownership,test-unit,build}.log` ;
- `rouges.log` — AC1, AC2, AC4 avant le code (commit `4531669f8`) ;
- `conversion-erreurs-1.txt`, `conversion-erreurs-2.txt` — le relevé du compilateur (sites à convertir) ;
- `lot-00` à `lot-02`, `fichiers-tests.txt` — le partage des fichiers de tests entre les agents ;
- `garde-releve-1.log` — la garde rouge sur le code converti, avant la mise à jour de sa table (12 sites neufs, 4 évanouis) ;
- `verts-cibles.log` — la garde, `audit_label_registry` et les tests ciblés, verts ;
- `clippy-1.log`, `manuels-make.log` ;
- `mut/M2p.log`, `mut/M3.log`, `mut/M7-M8-M9.log` ;
- `gate-db-reset.log`, `gate-test-fast.log`, `gate-ci.log` — gates du passage en `review`.

### Completion Notes List

**Base** (étape A). Fusion de `story/35-3-comptes-designes-type` achevée (commit `4531669f8`) : onze conflits de texte dont le côté
35-3 était vide (contenu déjà dans la base par la 35-1), côté de la base gardé ; `sprint-status.yaml` : 35-2a et 35-2b gardées à
`done` ; PDF pris de notre côté puis régénérés (`make -B user admin`). Comparaison à l'intégration à blanc `e2e1c39c4` : écarts
limités aux stories absentes (35-4, solde d'ouverture #1025-#1028, #1000) et à la 35-5c (C-e358a2-1 à -3). Gates de la base, au
commit `4531669f8` : `test-fast --ci` voie 2 **5439 passés, 0 échec, 13 ignorés** (fmt et clippy compris, `base-test-fast.log`) ;
frontend `check` 0 erreur, `lint-i18n-ownership` PASS, Vitest **3265** (234 fichiers), `build` vert (`base-front-*.log`).

**Étape 0** — lignes relues sur la base `4531669f8` (la fiche cite `main` `67efc51f`) :
- `journal_entries.rs` : `create` `:195`, `create_in_tx` `:242`, `create_in_tx_inner` `:281` (trace `:562`) ; `reverse_in_tx`
  `:2564`, `reverse_owned_in_tx` `:2646`, `reverse_at_next_fiscal_year_start_in_tx` `:2697`, `reverse_in_tx_inner` `:2816` (trace
  `journal_entry.reversed` `:3036`, lettrage `reversal` `:3085`), `reverse` `:3116`, `reverse_at_next_fiscal_year_start` `:3142` ;
  `update` `:1438`, `delete_by_id` `:1756`, `delete_in_tx` `:1858` ; et `create_opening_entry` `:655`, que la fiche ne nomme pas
  (C-e358a2-5) ;
- routes : `routes/journal_entries.rs:458`, `:497`, `:646`, `:708`, `:757` ; `routes/reconciliation.rs:2820`, `:3812`, `:4260`,
  `:5130`, `:5596` (comme la fiche) ; `reconciliation_cancel.rs:743` (`:723` à la fiche) ;
- autres appelants : `credit_notes.rs:703`, `invoice_settlements_write.rs:337`, `:700`, `:1026`, `invoices.rs:1707`, `:2424`,
  `opening_complement.rs:719`, `supplier_invoices.rs:1422`, `:1432` (retour en brouillon), `:1782`, `:2166` (dans `pay_in_tx_as`,
  qui a l'acteur : C-e358a2-4), `:2634`, `:2921`, `routes/opening_balances.rs:473` (`create_opening_entry`),
  `kesh-seed/src/pieges.rs:521` ;
- **rayon**, relevé par le compilateur (`conversion-erreurs-2.txt`, après la conversion de la production) : 217 lignes d'erreur —
  `journal_entries.rs` (`mod tests`) 80, `accounts.rs` (`mod tests`) 2, tests d'intégration 135 dans 48 fichiers ; la fiche
  annonçait ≈ 240 appels et ≈ 60 fichiers. Au commit `6d68be121`, depuis la base : **131** `Acteur::utilisateur(` ajoutés dans
  `crates/*/tests` (garde exclue), **80** dans le `mod tests` de `journal_entries.rs` (81 lignes ajoutées, dont celle de
  `create_opening_entry`), **2** dans celui d'`accounts.rs`.

**Rouges** (`rouges.log`, commit `4531669f8`) : AC1 saisie par clé (`journal_entry.created` à `("user", None, 1)`), AC1
rapprochement manuel (`journal_entry.created` à `'user'` ; `reconciliation.manual_matched` déjà à la clé), AC2 (`lettering.created`
de la paire `reversal` à `'user'` — après l'assertion de son existence, verte) ; le témoin d'AC4 vert, comme attendu.

**Livré** :
- `journal_entries.rs` : `create`, `create_in_tx`, `create_in_tx_inner`, `reverse`, `reverse_in_tx`, `reverse_owned_in_tx`,
  `reverse_at_next_fiscal_year_start[_in_tx]`, `reverse_in_tx_inner`, `update`, `update_in_tx`, `delete_by_id`, `delete_in_tx`
  prennent `acteur: Acteur` ; `journal_entry.created`, `.reversed`, `.updated`, `.deleted` par `NewAuditLogEntry::par` ; le
  lettrage `reversal` reçoit l'acteur (écart R3 levé, doc-comment de `letterings::Actor` corrigé) ;
- routes du journal : `Acteur::from(&current_user)` (calculé hors des fermetures de rejeu pour `update` et `delete`) ;
- les six sites qui ont la clé : `accept_one_invoice`, `accept_one_split`, `accept_one_rule`, `post_manual_once`,
  `post_split_once` (`Acteur { user_id, api_key_id: actor_api_key_id }`), `reconciliation_cancel.rs` genre `Entry` ; plus
  `supplier_invoices::pay_in_tx_as` (C-e358a2-4) ;
- les autres appelants de production passent `Acteur::utilisateur(user_id)` ; le retour en brouillon fournisseur passe l'`acteur`
  entier à `delete_in_tx` ;
- la garde : 4 entrées retirées (`create_in_tx_inner` `User`, `reverse_in_tx_inner` `User` et `ApiKeyNone`, `invoices::unvalidate`
  `DeleteInTxNone`), 12 ajoutées (`Utilisateur`) ; raison `Fixture` déclarée, référence close `pieges::semer` (liste vide : seuls
  des tests l'appellent) ; doc-comment : motif 5 sans site, angle mort 1 recompté (23 lignes de paramètre) ;
- AC1 : `an_entry_posted_by_an_api_key_is_attributed_to_the_key` (saisie, clé puis session) et
  `post_manual_by_an_api_key_attributes_the_entry_to_the_key` ; **aucun cas de `POST /reconciliation/accept`** n'est monté : les
  trois sites `accept_one_invoice`, `accept_one_split`, `accept_one_rule` sont couverts par la garde (aucune entrée `Utilisateur`
  pour `routes/reconciliation.rs`) et M9 le prouve pour l'un d'eux ;
  *(annoté le 2026-11-14, remédiation P1, A1 = E1 : le montage tenait — les cas `invoice`, `supplier_invoice`, `rule`, `split` et
  `POST /reconciliation/split` par clé sont montés et assertés depuis le commit `ec8117bcc` ; la garde ne voyait pas M10/M11)*
- AC2 : `both_reversals_by_an_api_key_are_attributed_to_the_key` (du jour et à l'exercice suivant, compte `1100` lettrable ;
  `lettering.created`, `journal_entry.reversed`, `journal_entry.created` de l'inverse) ; modification et suppression par clé :
  `a_read_write_key_modifies_and_is_traced_as_the_key`, `a_read_write_key_deletes_and_is_traced_as_the_key`, verts ;
- AC4 : `la_validation_par_cle_ecrit_encore_l_ecriture_au_nom_du_createur` (limite assertée) ; les tests de dépôt existants verts ;
  l'AC3 de la 35-8a1 vert (dans le gate) ;
- AC6b : `admin-manual.tex` (item *Journal d'audit*), `user-manual.tex` (tableau du journal d'audit), PDF régénérés
  (`manuels-make.log`) et contrôlés aplatis ; CHANGELOG, *Corrigé*, `refs #431`.

**La table de la garde, recomptée depuis la source** (`git show HEAD:…/audit_acteur_garde.rs`, `awk` sur les lignes `("crates/`
du bloc `SITES`) : **93 entrées, 98 sites** — `AConvertir` 53/54, `AdminSansCle` 27/29, `PreAuth` 4/4, `Systeme` 2/3, `Definition`
3/3, `Restauration` 1/2, `Mcp` 2/2, `Fixture` 1/1 ; par motif, `User` 71/76, `ApiKeyNone` 10/10, `Utilisateur` 12/12,
`DeleteInTxNone` 0. Raccord à la 35-8a1 (85/90) : − 4 entrées et 4 sites, + 12 et 12. Aucune entrée `Utilisateur` pour
`routes/reconciliation.rs`, `routes/journal_entries.rs`, `repositories/reconciliation_cancel.rs` (AC5b). `REFERENCES` : 51
fonctions (50 + `pieges::semer`), références inchangées.

**Mutations**, sur l'arbre commité `6d68be121`/`a9bf89787`, chacune par l'outil Edit, restaurée par `git checkout --` puis `touch`,
« restauré : 0 ligne » au journal :

| Mutation | Tuée par |
|---|---|
| M2' `create_in_tx_inner` trace par `NewAuditLogEntry::user(acteur.user_id, …)` | la garde (`create_in_tx_inner / User`, site neuf), AC1 saisie, AC1 rapprochement, AC2 (`mut/M2p.log`) |
| M3 lettrage `reversal` rendu à `api_key_id: None` | la garde (`reverse_in_tx_inner / ApiKeyNone`), AC2 (`mut/M3.log`) |
| M7 `post_manual_once` passe `Acteur::utilisateur(user_id)` | la garde (`post_manual_once / Utilisateur`), AC1 rapprochement (`mut/M7-M8-M9.log`) |
| M8 `reconciliation_cancel.rs` genre `Entry` passe `Acteur::utilisateur(user_id)` | la garde (`cancel_in_tx / Utilisateur`) |
| M9 `accept_one_split` passe `Acteur::utilisateur(user_id)` | la garde (`accept_one_split / Utilisateur`) |

M7, M8 et M9 ont été jouées **ensemble** (un seul passage de compilation) : la garde nomme chaque site séparément, et seul M7
touche le rapprochement manuel ; aucune mutation ne survit.
*(Annoté le 2026-11-14, remédiation P1, A4 : le journal de M2' finit par « restauré : 3 écart », suivi d'une annotation —
« git diff -- crates vide (0 ligne) ; les écarts ci-dessus sont les deux PDF régénérés » ; « 0 ligne » ci-dessus vaut pour
`crates`, non pour l'arbre entier. A5 : seule la première assertion d'AC2 — `lettering.created` de la contre-passation du jour —
a été vue rouge au `rouges.log` ; `journal_entry.reversed` et la branche de l'exercice suivant ne l'ont pas été (le test, lui,
est tué par M2' et M3 : `mut/M2p.log`, `mut/M3.log`).)*

**Gates du passage en `review`**, au dernier commit de code `6d68be121` (les commits suivants ne touchent ni `crates` ni
`frontend` ni `scripts` : `git diff --stat 6d68be121 HEAD -- crates frontend scripts` vide) :

| Gate | Résultat | Journal | Commit |
|---|---|---|---|
| `cargo fmt --check`, `clippy -D warnings` (workspace) | verts | `clippy-1.log` ; `gate-test-fast.log` (« ▶ cargo fmt --check », « ▶ cargo clippy ») | `6d68be121` |
| `scripts/test-fast.sh --ci`, voie 2, bases remises à neuf (`gate-db-reset.log`) | **5443 passés, 0 échec, 13 ignorés** (+4 sur la base : les quatre tests neufs) | `gate-test-fast.log` | HEAD `a9bf89787`, code `6d68be121` |
| `scripts/gate-ci.sh kesh_ci_e358a2`, voie 1 | **5450 passés, 0 échec, 17 ignorés** | `gate-ci.log` | idem |
| frontend | non relancé : rien ne change sous `frontend/` ; vert sur la base (`base-front-*.log`) | — | `4531669f8` |
| E2E complet | **non lancé** : à la clôture (consigne de l'Epic 35) | — | — |
| captures | aucune : aucun contrôle visible ne change | — | — |

**Ce qui n'a pas tourné** : l'E2E (à la clôture) ; le frontend au commit de code (non touché ; joué sur la base) ; aucune capture
(aucun contrôle visible).

**Écarts à la consigne Edit** : le registre et le prompt de revue créés par l'outil Write (l'outil Edit ne crée pas de fichier) ;
`cargo fmt` a reformaté ; mes `replace_all` ont touché à tort 37 destructurations `let (…, admin_user_id, today)` du `mod tests`,
rétablies par Edit après le compilateur. Aucun `sed -i` ni script qui écrit dans le dépôt. Écart de la base, déclaré par
l'orchestrateur : marqueurs du CHANGELOG de la fusion 35-8a1 retirés par `sed -i`.

**Limites voisines**, à suivre en issues (une par problème, demandées au rapport) : aucune neuve au-delà de #1130 à #1132.

### File List

- `crates/kesh-db/src/repositories/journal_entries.rs` — signatures à l'`Acteur`, traces par `par`, lettrage `reversal` ; `mod tests`
- `crates/kesh-db/src/repositories/{credit_notes,invoice_settlements_write,invoices,opening_complement,reconciliation_cancel,supplier_invoices}.rs` — appelants
- `crates/kesh-db/src/repositories/letterings.rs` — doc-comment de `Actor`
- `crates/kesh-db/src/repositories/accounts.rs` — `mod tests`
- `crates/kesh-api/src/routes/journal_entries.rs`, `crates/kesh-api/src/routes/reconciliation.rs` — l'acteur passé
- `crates/kesh-seed/src/pieges.rs` — `Acteur::utilisateur`
- `crates/kesh-api/tests/audit_acteur_garde.rs` — table, raison `Fixture`, `REFERENCES`, doc-comment
- `crates/kesh-api/tests/journal_entry_reversal_e2e.rs` — AC1 (saisie), AC2 ; conversion
- `crates/kesh-api/tests/reconciliation_manual_e2e.rs` — AC1 (rapprochement)
- `crates/kesh-api/tests/invoice_unvalidate_e2e.rs` — AC4 (témoin de la limite)
- 47 autres fichiers de tests de `crates/kesh-{api,db,report}/tests` (dont `kesh-db/tests/support/lettering_documents.rs`) —
  conversion mécanique (`git diff --stat 4531669f8 6d68be121 -- 'crates/*/tests/*'`)
- `docs/manual/fr/admin-manual.tex`, `.pdf` ; `docs/manual/fr/user-manual.tex`, `.pdf` ; `CHANGELOG.md`
- `_bmad-output/implementation-artifacts/e35-choix-autonomes-e358a2.md` — **neuf**, registre
- `_bmad-output/implementation-artifacts/35-8a2-audit-acteur-ecritures-review-prompt-p1.md` — **neuf**, prompt de la revue P1
- `_bmad-output/implementation-artifacts/sprint-status.yaml`, cette fiche

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
- 2026-11-14 — **Base** (Opus 5.5) : fusion de la 35-3 achevée (`4531669f8`, onze conflits de texte réglés côté base — côté 35-3
  vide —, PDF régénérés ; C-e358a2-1 à -3) ; gates de la base verts (`test-fast --ci` 5439/0/13, frontend complet, Vitest 3265).
- 2026-11-14 — **Développement** (Opus 5.5, voie 2 ; conversion des tests par trois agents Sonnet, C-e358a2-6).
  - Étape 0 : lignes relues sur `4531669f8`, rayon relevé par le compilateur (217 lignes d'erreur ; 131 + 80 + 2 sites convertis),
    détail au Dev Agent Record.
  - Rouges d'AC1, AC2 et témoin d'AC4 au commit `3be2d577c` (`rouges.log`).
  - Code au commit `6d68be121` : signatures de `journal_entries.rs` à l'`Acteur`, six sites qui ont la clé et `pay_in_tx_as`
    (C-e358a2-4), `create_opening_entry` inchangée (C-e358a2-5), autres appelants à `Acteur::utilisateur`, garde à 93 entrées / 98
    sites (`Utilisateur` 12, motif 5 à zéro, raison `Fixture`), manuels et CHANGELOG ; PDF au commit `a9bf89787`.
  - Mutations M2', M3, M7, M8, M9 tuées (M7 à M9 jouées ensemble, nommées séparément par la garde).
  - `test-fast --ci` 5443/0/13 (voie 2) ; `gate-ci` 5450/0/17 (voie 1) ; frontend non touché ; **E2E à la clôture**.
  - Écart : mes `replace_all` ont touché 37 destructurations, rétablies (Dev Agent Record). Statut : `review`. Prompt de la revue
    P1 : `35-8a2-audit-acteur-ecritures-review-prompt-p1.md`.
- 2026-11-14 — **Revue de code P1** (Opus, trois lentilles, rapports `kesh-gate-logs/e358a2-review-p1-{B,E,A}.md`) : lentille B
  1 MEDIUM / 7 LOW, lentille E 2 MEDIUM / 3 LOW, lentille A 2 MEDIUM / 5 LOW ; 0 CRITICAL, 0 HIGH ; **deux MEDIUM distincts**
  (M-A = B1 = A2 = E2, M-B = A1 = E1).
- 2026-11-14 — **Remédiation P1** (Opus 5.5, voie 2). Code au commit `ec8117bcc`, compte rendu et PDF au commit suivant.
  - **M-A — textes contradictoires.** Chemins relus au code avant d'écrire : règlement client par rapprochement,
    `routes/reconciliation.rs:2823-2826` (`accept_one_invoice` : `Acteur { user_id, api_key_id: actor_api_key_id }` passé à
    `create_in_tx`) ; paiement fournisseur par rapprochement, `:2118-2125` (`letterings::Actor { user_id, api_key_id:
    actor_api_key_id }` vers `accept_one_supplier_invoice`) puis `:3426` (`pay_in_tx_as(…, actor)`) et
    `supplier_invoices.rs:2156` (`actor` passé à `create_in_tx`) ; règlement saisi sur la facture, `lib.rs:770-771`
    (`POST /api/v1/invoices/{id}/settlements` → `settle_invoice_handler`) → `invoice_settlements_write::settle_invoice`,
    `:342` (`Acteur::utilisateur(user_id)`) ; solde du reste, `lib.rs:780` → `write_off_invoice`, `:706` (`Acteur::utilisateur`) ;
    paiement par la fiche, `lib.rs:672-673` → `supplier_invoices::pay` → `pay_in_tx`, `supplier_invoices.rs:1954-1958`
    (`api_key_id: None`) ; par un lot, `payment_batches.rs:584` (`pay_in_tx`) ; dé-rapprochement,
    `reconciliation_cancel.rs:713-739` (règlements : `cancel_settlement_in_tx(…, user_id, …)`) et `:744-750` (genre `Entry` :
    `Acteur { user_id, api_key_id: actor_api_key_id }`). Récrits : `CHANGELOG.md` (entrées 35-8a1 et 35-8a2), item *Journal
    d'audit* du manuel d'administration (ses deux phrases), tableau du journal d'audit du manuel utilisateur — « toutes celles
    qu'un rapprochement bancaire passe (y compris le règlement d'une facture client et le paiement d'une facture fournisseur que le
    rapprochement enregistre) » d'un côté ; « un règlement saisi sur la facture », « paiement depuis la fiche ou par un lot de
    paiement » de l'autre (C-e358a2-7). Symptôme greppé (`validation, paiement, annulation`, `un avoir, un règlement`,
    `qu'un rapprochement`) : seuls restent la fiche (AC6b, annotée, non réécrite), le prompt P1 et le registre (relevés datés) ;
    manuels de/en/it et catalogues : aucune occurrence de #431. PDF régénérés (`p1-manuels-make.log`, `make -B user admin`), contrôlés
    aplatis : le texte du PDF reprend le `.tex`.
  - **M-B — écritures d'un rapprochement par clé testées.** `journal_entry.created` → `('api_key', Some(clé), créateur)`, trace
    **unique** (`common::audit_actor_unique`, `fetch_all` puis longueur 1), ajouté à `accept_letters_a_fully_settled_invoice`
    (facture client, l'encaissement et non la vente) et à `an_api_key_accepts_a_supplier_invoice_and_is_named_everywhere`
    (C-e358a2-4) ; test neuf `an_api_key_rule_and_split_entries_are_attributed_to_the_key` (montage `setup_301b` : une règle et une
    ventilation proposées acceptées par la clé, puis `POST /reconciliation/split` par la même clé ; trois écritures).
  - **Mutations** (`kesh-gate-logs/e358a2/p1-mut/`, chacune par l'outil Edit, restaurée par `git checkout --` puis `touch`,
    « restauré : 0 ligne(s) de diff sous crates » au journal) :

    | Mutation | Tuée par | Garde |
    |---|---|---|
    | M10a — bras `split` d'`accept_one`, `None` à la place d'`actor_api_key_id` | test neuf, « écriture du rapprochement split » (`M10a.log`) | verte (angle mort 1, #1130) |
    | M10b — bras `rule`, idem | test neuf, « écriture du rapprochement rule » (`M10b-M12-M13.log`) | verte |
    | M11 — `post_split_once` : `let actor_api_key_id: Option<i64> = None;` | test neuf, « écriture du rapprochement /reconciliation/split » (`M11-M13p.log`) | verte |
    | M12 — `pay_in_tx_as` passe à `create_in_tx` un acteur dont la clé est jetée (`api_key_id.filter(\|_\| false)`, forme que la garde ne voit pas) | « écriture de règlement fournisseur » (`M10b-M12-M13.log`) | verte |
    | M13 — bras `invoice`, `None` | `accept_letters_…`, mais par l'assertion antérieure de `lettering.created` (`M10b-M12-M13.log`) | verte |
    | M13' — `accept_one_invoice` : l'écriture seule perd la clé (`.filter(\|_\| false)`), le lettrage la garde | « écriture d'encaissement » (`M11-M13p.log`) | verte |

    M10b, M12 et M13 jouées ensemble, M11 et M13' ensemble : chaque mutation d'un même passage touche un test distinct. Aucune ne
    survit. Toutes passent la garde : sans ces tests, elles survivaient (ce que disaient A1 et E1).
  - **LOW traités** : B2 (exemple de `retry_on_deadlock` à l'`Acteur`) ; B3 (commentaire du `mod tests` de `journal_entries.rs`) ;
    B4 = A7 = E4 (`api-external.md:677`, `:713`, `:739` et doc de `pay_in_tx_as` citent l'écriture) ; B6 = A6 = E5 (unicité :
    `common::audit_actor_unique`, employé par `acteur_de`, le témoin d'AC4, les tests de `reconciliation_e2e.rs` ; forme en ligne
    dans `reconciliation_manual_e2e.rs`, binaire sans `common`) ; B7 (motif 5 dit « forme morte », échantillon `g` de la sonde (h)
    à la signature d'aujourd'hui) ; B8 (couvert par M-B) ; E3 (commentaires de `reconciliation_e2e.rs` corrigés ; le test du
    dé-rapprochement par clé asserte `journal_entry.reversed` de l'écriture rapprochée et `journal_entry.created` de l'inverse —
    assertions non vues rouges par mutation : M8 de la passe de développement porte sur ce site) ; A3 (déclaré : la saisie d'AC1 est
    jouée par une clé de l'administrateur ; l'attribution ne dépend pas du rôle — `Acteur::from(&current_user)` —, et le cas d'un
    Comptable est celui du rapprochement manuel, `setup_full(…, Role::Comptable)`) ; A4, A5 (annotés au Dev Agent Record).
  - **LOW non traité** : B5 (les cinq littéraux `Acteur { user_id, api_key_id: actor_api_key_id }` de `routes/reconciliation.rs`) —
    construire l'`Acteur` une fois dans `post_accept`/`post_manual_once`/`post_split_once` change la signature des six fonctions
    `accept_one*` et déplace les sites de la garde ; c'est un refactor, hors d'une remédiation (C-e358a2-8) ; issue à ouvrir par
    l'orchestrateur.
  - **Gates** au commit de code `ec8117bcc` : `cargo fmt` ; `clippy -D warnings` workspace vert (`p1-clippy.log`, sur l'arbre du
    commit avant commit) ; tests ciblés 104/104 (`p1-cibles.log`) ; base `kesh_e358a2` remise à neuf (`p1-db-reset.log`) ;
    `test-fast --ci` voie 2 (fmt et clippy compris) : **5444 passés, 0 échec, 13 ignorés**, `rc=0` (`p1-test-fast.log` ; +1 sur
    le passage en `review` : le test neuf). **Non lancés** : `gate-ci` et E2E, à la clôture (consigne du
    2026-11-13) ; frontend non touché.
  - Écarts aux consignes : le prompt de la P2 ciblée créé par l'outil Write (fichier neuf) ; un script de mutation dans le
    répertoire de travail temporaire de la session (hors dépôt), qui restaure par `git checkout --` et `touch` ; les journaux
    redirigés par `>` vers `kesh-gate-logs/e358a2/` (`p1-*.log`, `p1-mut/*.log`). Prompt de la P2 ciblée :
    `35-8a2-audit-acteur-ecritures-review-prompt-p2-ciblee.md`.
  - *Annotation du 2026-11-14, à la clôture (LOW C3 de la P2 ciblée)* : l'entrée ci-dessus dit les mutations « chacune par l'outil
    Edit » ; elles ont en fait été posées par le script de mutation hors dépôt nommé aux écarts, et jouées par groupes (M10b, M12
    et M13 ensemble ; M11 et M13' ensemble), non une à une. L'entrée n'est pas réécrite.
- 2026-11-14 — **Revue de code P2 ciblée** (Sonnet, une lentille, le seul commit `ec8117bcc` ;
  `kesh-gate-logs/e358a2-review-p2-ciblee.md`) : **0 au-dessus de LOW**, 3 LOW (C1 à C3). Les deux MEDIUM de la P1 remédiés.
  Boucle de revue terminée.
- 2026-11-14 — **Clôture** (Opus 5.5, worktree `kesh-e358a2`). Synthèse finale de la boucle :
  - **Tendance** : P1 (Opus, trois lentilles) — B 1 MEDIUM / 7 LOW, E 2 MEDIUM / 3 LOW, A 2 MEDIUM / 5 LOW, soit **deux MEDIUM
    distincts** (M-A textes, M-B tests d'exécution), 0 CRITICAL, 0 HIGH → remédiation `ec8117bcc` → P2 ciblée (Sonnet) **0 MEDIUM**,
    3 LOW. Modèles : Opus en P1, Sonnet en P2 ; aucun reclassement, aucun finding réfuté.
  - **LOW de la P2** : C1 appliqué, commit `3cbea0e05` (commentaire de l'étape (g bis) d'`accept_one_invoice`,
    `routes/reconciliation.rs:2982-2983`, parenthèse « seul geste qui la porte, AC10 » retirée : la clé est aussi passée à
    `create_in_tx`) ; C2 joué — mutation **M14**, `reconciliation_cancel.rs:746` (bras `Entry`) `api_key_id:
    actor_api_key_id.filter(|_| false)`, forme que la garde ne voit pas : **tuée** par `an_api_key_cancels_and_is_named_in_the_audit`,
    `reconciliation_e2e.rs:5412`, « assertion `left == right` failed: journal_entry.reversed — left: ("user", None, 1), right:
    ("api_key", Some(1), 1) » ; l'assertion `journal_entry.created` de l'inverse, qui suit, n'est pas atteinte par cette mutation
    (l'acteur est le même pour les deux traces, `reverse_in_tx(…, acteur, …)`) ; restaurée par Edit inverse puis `touch`,
    `git status --porcelain` vide (`kesh-gate-logs/e358a2/clot-mut-M14.log`) ; C3 annoté ci-dessus. LOW non traité de la P1 : B5,
    issue **#1142**.
  - **Dernier commit de code : `3cbea0e05`** (un commentaire de production).
  - **Gates au commit `3cbea0e05`** :
    - `cargo fmt --all -- --check` vert avant le commit ;
    - `scripts/gate-ci.sh kesh_ci_e358a2`, voie 1 : **5451 passés, 0 échoué, 17 ignorés** (5450 au passage en `review` + le test
      neuf de la remédiation P1) (`kesh-gate-logs/e358a2/cloture-gate-ci.log`) ;
    - E2E complet, voie 2, base `kesh_e2e_e358a2` recréée (91 migrations, seed), frontend reconstruit, backend sur le port 3416
      (dossiers sous `target/e2e-c/`) : **405 passés, 7 échecs, 18 ignorés** ; les sept échecs sont exactement ceux de la KF-029
      (#97) au tableau de `docs/testing.md` § « Les échecs attendus » (`mode-expert.spec.ts:26`, `:41`,
      `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150`) ; aucun échec hors liste
      (`kesh-gate-logs/e358a2/cloture-e2e.log`, backend `cloture-e2e-backend.log`, arrêté par son PID, port libre après).
  - Consigne Edit : tenue (commentaire, mutation et sa restauration, fiche, sprint-status). Le script de l'E2E est un fichier du
    répertoire de travail de la session, hors dépôt ; les dossiers E2E sont sous `target/e2e-c/` du worktree, comme le veulent les
    consignes communes (et non sous `/tmp`).
  - Statut `done`.

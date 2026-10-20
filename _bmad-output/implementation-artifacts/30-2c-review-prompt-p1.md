# Prompt — revue de code P1, Story 30-2c (le document joint, détaché et consulté)

*Versionné le 2026-10-20. Trois lentilles (B, E, A), contexte frais chacune, lecture seule. Modèle : **Sonnet**
(passe complète — jamais Haiku pour une passe complète, D6 de la rétrospective de l'Epic 25).*

Dépôt (worktree) `/home/gcorbaz/devel/kesh-30-2`, branche `story/30-2c`, partie de `1aca02d2` (merge de la 30-2b,
PR #691) et **reportée sur `4663f60b`** (merge de la 30-3, PR #692). Langue du rapport : français.

**Le diff à revoir** : `git -C /home/gcorbaz/devel/kesh-30-2 diff 4663f60b 67ac15e0 -- . ':(exclude)_bmad-output'` —
`f8f81f6d` porte le code, les tests et les E2E ; `67ac15e0` (**dernier commit de code**) porte la documentation, les PDF
et un doc-comment de test ; `ea986af6` les PDF régénérés après le report. Les commits suivants ne portent que la fiche,
`sprint-status.yaml` et ce prompt. (Avant le report, ces commits étaient `0d18652d` et `c9e6fea6` : les journaux de la
campagne de mutations portent `0d18652d`, de code identique à `f8f81f6d` hors les changements de la 30-3.)
⚠️ Pathspec `':(exclude)…'`, **non** `':!…'`.

⚠️ **Reprise.** Le développement a été interrompu le 2026-10-16 puis repris le 2026-10-20 : le dépôt (`attach_in_tx`,
`detach_in_tx`, `find_source_document_of_entry`), les routes, `errors.rs` et les tests Rust sont du premier agent,
relus et conservés par le second (Dev Agent Record de la 30-2c). Relis-les comme le reste, sans présumer qu'ils ont été
revus.

Fiche : `_bmad-output/implementation-artifacts/30-2-documents-factures-fournisseur.md` — **partie c seulement** :
AC18 à AC25 ; tâches T-c1 à T-c6 ; § *Gardes existantes* (lignes « c »), *E2E de parcours* (E4 à E7), *Mutations*
(M-c1 à M-c12), *Angles morts assumés* ; et le **Dev Agent Record** de la 30-2c. **Ajouts de l'orchestrateur** : la fiche
montre `creditorQrIban` (C-30-2-47) ; AC25 (`paidAt` formaté). Choix : `epic-30-choix-autonomes-302.md`, C-30-2-1 à 44
et **C-30-2-45 à 51** (développement de la 30-2c — dont **C-30-2-46, la bascule de l'AC19 sur la sentinelle de la
société**, prise parce que le test déterministe a rougi sur le vrai schéma). Règles : `CLAUDE.md` du dépôt —
§ *Pattern batch*, § *Recompter ses propres comptes rendus*, § *Propagation post-patch*, § *Le prompt d'une passe doit
NOMMER le manuel*, § *Une passe qui ne déclare pas ses axes exercés ne compte pas*. Issue : #685 — **aucun mot-clé de
fermeture** dans un message de commit, même en prose : `git -C /home/gcorbaz/devel/kesh-30-2 log 4663f60b..HEAD
--format=%B | grep -niE "\b(close[sd]?|fix(e[sd])?|resolve[sd]?):?\s+#[0-9]+"` doit être vide (la PR, elle, portera
`closes #685`, posé par l'orchestrateur).

⛔ **Lecture seule stricte.** N'écris, ne modifie, ne crée aucun fichier hormis ton rapport. **Aucun** `git checkout`,
`git switch`, `git stash`, `git reset`, `git commit`, `git tag`, `git push`, `git rebase` — sous aucune forme. **Aucune
commande qui écrit, compile ou exécute** : ni `cargo`, ni `npm`/`npx`, ni `make`/`latexmk`, ni
`scripts/prepare-release.sh`, ni `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni `python3`, ni `node`, ni
`sed -i`, ni `perl`, ni `mariadb`/`docker`. Aucune commande `gh` qui écrit (`gh issue view` est permis). Permis :
`git diff`, `git show`, `git log`, `git grep`, `grep`, `sed -n`, `cat`, `ls`, `jq`, et
`pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '` **vers la sortie standard uniquement** — jamais vers un fichier.

⛔ **Données réelles — dépôt public.** N'ouvre rien hors du worktree et de `/home/gcorbaz/devel/kesh-gate-logs/`. Les
captures `kesh-gate-logs/302c-captures/302c-*.png` montrent des fixtures synthétiques : tu peux les ouvrir.

## Axes communs aux trois lentilles (chacune les déclare exercés ou non)

1. **Le rattachement** (`crates/kesh-db/src/repositories/imported_supplier_invoices.rs` `attach_in_tx`,
   `AttachRefusal` ; route `attach_source_document` / `attach_source_document_once`,
   `crates/kesh-api/src/routes/imported_supplier_invoices.rs`) : ordre des verrous document → facture → sentinelle
   `companies` → entrée d'unicité ; aucune lecture préalable du lien existant (C-30-2-21) ; la violation d'unicité
   traduite **seulement** si elle nomme `uq_imported_supplier_invoice` ; portée société des deux côtés (document **et**
   facture, IDOR) ; statut `to_complete` exigé ; tout statut de facture ; aucun champ de la facture modifié ; audit
   dans la transaction ; réponse relue après `COMMIT`. **La sentinelle (C-30-2-46)** : son motif est-il juste, sa place
   ne forme-t-elle pas d'autre cycle que celui nommé avec l'acceptation de la 30-1b (`pay_in_tx`, `cancel_in_tx`,
   `cancel_settlement_in_tx`, `complete_import_once`, `payment_batches::create`, le détachement) ? Le rejeu
   (`retry_app_on_deadlock`) couvre-t-il tout le corps ?
2. **Le détachement** (`detach_in_tx`, `detach_source_document_once`) : ne verrouille **que** la ligne du document
   (jamais la facture), `UPDATE` gardé par le statut, 0 ligne = erreur, `supplier_invoice_id = NULL` **et**
   `to_complete` ensemble, aucun fichier supprimé, 404 `SOURCE_DOCUMENT_NOT_FOUND` (inconnue, autre société, sans
   document), audit. Après détachement, un autre document se joint (M-c9).
3. **Les lectures** : `find_by_supplier_invoice_id_scoped` (exécuteur générique, `AND status = 'completed'`, lue dans
   l'instantané de `get_settlement_view`) ; `sourceDocument` de `SupplierInvoiceResponse` à **trois états**
   (`Option<Option<_>>`, `skip_serializing_if` sur l'extérieur ; absent de la réponse de création) — toutes les routes
   qui relisent la facture le calculent-elles, et aucune ne le rend `null` à tort ? `find_source_document_of_entry`
   (achat **et** règlement, société, `LIMIT 1`, règlement annulé) lue dans la transaction de `get_journal_entry`, et la
   garde lexicale `get_journal_entry_reads_in_one_transaction` paramétrée par `(module, lecteur)` : mord-elle vraiment
   sur une lecture au pool ? `JournalEntryDetailResponse` a-t-il un seul constructeur ? Lecture du document d'une
   facture montée parmi les lectures (`crates/kesh-api/src/lib.rs`), celle d'un document à traiter restée Comptable+ ;
   les deux montages du même chemin (`GET` dans un bloc, `POST`/`DELETE` dans l'autre) se fusionnent-ils sans
   recouvrement ni contournement du RBAC ?
4. **L'écran** — fiche `frontend/src/routes/(app)/supplier-invoices/[id]/+page.svelte` (nom du document, *Voir le
   document* = aperçu, *Télécharger*, *Détacher* Comptable+ avec la confirmation prescrite ; *Joindre un document* sur
   `null` seulement, rien sur `undefined` ; QR-IBAN ; `paidInfo` et ses deux clés), sélecteur
   `frontend/src/lib/features/imported-supplier-invoices/AttachDocumentPicker.svelte` (liste `to_complete`, badge, date
   `importDateOf`, montant du QR à côté du total, aperçu à lecture stable, confirmation, refus affiché sans fermer,
   liste vide), page d'écriture `frontend/src/routes/(app)/journal-entries/[id]/+page.svelte` (lien, aperçu, lecture
   qui suit la facture, aperçu refermé au changement d'écriture). Svelte 5 : `$derived.by` de la lecture, `{#key}`,
   révocation des URL `blob:` ; rôle Consultation ; double clic sur *Joindre* ou *Détacher* ; refus du détachement.
5. **i18n et gardes** : clés neuves ×4 locales (sept `supplier-invoices-*`, six `imported-supplier-invoices-attach-*`,
   trois `journal-entries-*`, `error-source-document-already-attached`, deux `audit-log-action-…-attached/-detached`),
   valeur changée `supplier-invoices-err-currency` ×4 et le message serveur de `CURRENCY_NOT_SUPPORTED` ; parité,
   vraies traductions, mêmes variables, replis égaux à fr-CH. `sitesTotal` 2251 → 2279, `CLES_RELEVEES` 328 → 341,
   `boutons-rouges-inventory` 13 → 14, `audit_route_registry.rs` (122 / 116 / 125 / 97, point (vi) « dix-sept »),
   `audit_labels.rs` : **recomptés depuis la source**.
6. **Les tests prouvent-ils ce qu'ils nomment.** Rust : `crates/kesh-db/tests/supplier_document_attachment.rs` (neuf
   tests, dont les tests de concurrence sans rejeu — C-30-2-45 : ils appellent `_in_tx`, non `_once`), cinq tests neufs
   de `crates/kesh-api/tests/inbox_import_e2e.rs` (section 30-2c, 34 → 39). Vitest : `supplier-document-page.test.ts`
   (fichier neuf, 14 tests),
   section 30-2c de `journal-entry-page.test.ts`, mock adapté de `supplier-settlement-page.test.ts`. E2E :
   `supplier-documents.spec.ts` (E4 → E5 → E7 en `describe.serial`, E6 ; E1 et E2 assertent désormais la fiche) et
   `reconciliation-supplier.spec.ts` (date au format suisse). La campagne (`kesh-gate-logs/302c-mutations.log`, motifs
   `kesh-gate-logs/302c-mut/*/`, journaux `302c-mut-<id>.log` ; ⚠️ le libellé « ERREUR-COMPILATION » du journal est un
   faux libellé annoté en fin de fichier — vérifie-le) : chaque « tuée » l'est-elle **pour la bonne raison** ? M-c11 n'est
   tuée que par `attach_waits_for_an_uncommitted_detach_then_succeeds` : est-ce une preuve suffisante ? Cherche la
   **mutation plausible qui resterait verte** (exemples : `sourceDocument` mis à `Some(None)` dans `from_parts` ;
   `skip_serializing_if` retiré ; `AND isi.company_id = si.company_id` retiré de la jointure ; le détachement sans
   `AND status = 'completed'` dans l'`UPDATE` ; la fiche qui affiche *Joindre* pour Consultation ; le sélecteur qui
   n'appelle pas `confirm`).
7. ⛔ **LA DOCUMENTATION — manuels et PDF, `api-external.md`, CHANGELOG, README.** Chaque phrase neuve ou modifiée
   dit-elle vrai **au code** ? Manuel utilisateur (`docs/manual/fr/user-manual.tex`) : § *Documents à traiter*
   (`sec:import-factures`) — paragraphe *Le document d'une facture* (voir, télécharger, détacher, joindre, rôle
   Consultation, aucun contrôle de montant, une facture = un document), l'avertissement du document à QR détaché,
   l'écriture d'achat ou de règlement, la note CHF ; renvoi #503 conservé. Manuel d'administration
   (`docs/manual/fr/admin-manual.tex`) : **§ *Passer à la 0.14.0*** (`sec:maj-0-14-0`), paragraphe *Intégrations par
   l'API* (routes neuves, `sourceDocument`, lecture ouverte). `docs/api-external.md` : section *Le document d'une facture
   fournisseur* (trois états, codes, rôles, angle mort) et paragraphe des interblocages rejoués — vérifie chaque code,
   statut et champ contre `errors.rs` et les routes, et la phrase « les réponses de `pay`, `cancel`,
   `settlement/cancel` … portent `sourceDocument` » contre le code. CHANGELOG `## [0.14.0] — Non publié` : bandeau
   (*Intégrations*), *Ajouté* (joindre / détacher, document depuis l'écriture), *Modifié* (lecture ouverte, date et
   QR-IBAN, message de devise) ; `[0.13.4]` intact. README l. 44 (« import lancé depuis l'écran » gardé, G5 de
   `textes_coherents.rs`). Contrôle les **PDF aplatis** (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' |
   tr -s ' '`, de même `admin-manual.pdf`, **vers la sortie standard**) — ⚠️ ligatures, apostrophe typographique « ’ » et
   césure (« authenti- fié ») rendent un `grep` naïf faux négatif. Les PDF sont-ils postérieurs à leur `.tex` ? Grepe la
   **valeur** du vocabulaire changé (« Retirez le document de la liste. » et ses traductions, « Voir la facture
   d'origine », « Facture réglée. » suivi d'une date brute) sur `docs`, `README.md`, `website`, `crates`, `frontend`.

## Lentilles

- **B — Blind Hunter : bugs et états**, sans présupposer que la fiche a raison. Verrous et cycles (axe 1), la course
  rattachement / complétion / détachement, la relecture après `COMMIT` (réponse d'erreur sur un succès), la sécurité
  (IDOR sur les deux identifiants, lecture ouverte : un rôle Consultation peut-il lire le document d'**une autre
  société** ou un document **à traiter** par un détour ?), Svelte 5 (états du sélecteur, `invoice` remplacé par la
  réponse, aperçu qui lit un document détaché ailleurs).
- **E — Edge Case Hunter : bords, et les tests prouvent-ils ce qu'ils nomment.** Facture annulée ou payée, document
  `discarded` ou `completed`, document détaché puis re-complété (seconde facture), règlement annulé, facture sans
  numéro, document au fichier manquant (410) dans l'aperçu de la fiche et de l'écriture, deux onglets (joindre ici,
  détacher là), sélecteur ouvert pendant qu'un import ajoute des documents, `paidAt` nul ou au format inattendu ;
  rejoue mentalement la campagne de mutations (axe 6).
- **A — Acceptance Auditor : conformité, décomptes, documentation.** Chaque AC (AC18 à AC25) et les deux ajouts de
  l'orchestrateur contre le code : satisfait, partiel, absent ; T-c1 à T-c6 ; M-c1 à M-c12 présentes et conformes (M-c7
  déclarée équivalente) et les mutations ajoutées (M-c13 à M-c16) ; les écarts à la fiche sont-ils **tous** déclarés au
  Dev Agent Record (C-30-2-45 : `_in_tx` au lieu de `_once` ; C-30-2-46 : la bascule) ; les chiffres de gates et d'E2E
  déclarés correspondent-ils aux journaux `kesh-gate-logs/302c-*.log` ; échecs E2E jugés fichier par fichier contre
  `docs/testing.md` § *Les échecs attendus* ; `sprint-status.yaml` : clé `30-2c-document-joint-et-consulte` en
  `review`, commentaire daté ; **recompte depuis la source** chaque décompte du Dev Agent Record (tests neufs par
  fichier aux deux bornes, clés, mutations, gates) et vérifie son **périmètre**.

## Ce que tu rends

Ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/302c-review-p1-<B|E|A>.md` (ta lettre), et rien d'autre nulle
part :

- findings numérotés `B1…` / `E1…` / `A1…`, chacun avec sévérité (CRITICAL / HIGH / MEDIUM / LOW), `fichier:ligne`,
  **preuve** (sortie `grep -nF` ou `sed -n` copiée pour toute affirmation de présence ou d'absence), correctif
  proposé ;
- **la liste des axes réellement exercés ET de ceux qui ne l'ont pas été** (axes 1 à 7 et ceux de ta lentille) — un
  « 0 finding » sans cette liste ne compte pas comme passe ;
- un décompte final `CRITICAL/HIGH/MEDIUM/LOW`.

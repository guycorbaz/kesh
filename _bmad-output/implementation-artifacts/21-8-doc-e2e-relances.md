# Story 21.8: Documentation & E2E round-trip des rappels débiteurs

Status: ready-for-dev

<!-- Créée 2026-07-20 par bmad-create-story. Dernière story de l'Epic 21 (clôture). Cartographie ground-truth par 3 agents Explore (manuels LaTeX / couverture E2E round-trip / refactor A4 credit-notes). Story DOC + TEST : aucune logique métier nouvelle. Consomme 21-3..21-7 (toute la feature rappels + balance âgée est livrée). -->

## Story

En tant que **fiduciaire / PME suisse (utilisateur) et administrateur (DevOps)**,
je veux **une documentation complète du cycle de rappels débiteurs et de la balance âgée dans les manuels, un test E2E qui valide le parcours de bout en bout, et des fixtures de test propres**,
afin de **savoir configurer et utiliser les relances (bases légales, frais, LPD comprises), avoir la garantie que le cycle complet fonctionne, et une base de tests maintenable pour la suite**.

## Contexte

L'Epic 21 a livré tout le cycle débiteurs : conditions de paiement (#245), TTC canonique (#246), socle + réglages rappels (21-3/21-4), éligibilité + envoi (21-5), page Rappels + intégrations fiche/dashboard (21-6a/b/c), balance âgée (21-7). **Il manque la clôture : documentation utilisateur/admin, un test E2E round-trip, et le nettoyage de dette A4.**

**21-8 est une story DOC + TEST — aucune logique métier, aucun endpoint, aucune migration.** Trois volets :

1. **Manuels FR** (admin + user) : documenter les réglages de rappels, le cycle de relance, les frais/CGV, le recommandé/mise en demeure, la balance âgée, et la rétention LPD (`sent_to`, CO 958f 10 ans).
2. **E2E round-trip Playwright** : un test qui chaîne une **seule facture** à travers le cycle complet (config niveaux → facture échue → liste à rappeler → envoi unitaire + lot capturés MockMailer → historique → suspension → balance âgée).
3. **A4 (dette héritée Epic 20)** : re-pointer `credit-notes.spec.ts` sur les fixtures partagées `api-fixtures.ts`.

### Décisions figées

- **D-8a — Bump des macros version = étape RELEASE, pas 21-8.** Le gate 4-bis (`CLAUDE.md`) impose de bumper `\keshVersion`/`\keshReleaseDate`/`\keshTargetRelease` **à la création de la release**. 21-8 met à jour le **contenu** des manuels + régénère les PDF, mais **ne touche PAS les 3 macros version** (elles restent à `0.6.0`/`v0.6` ; le release Epic 21 les portera à la version publiée + re-régénérera les PDF). Rationale : la date de publication n'est pas connue au moment de 21-8 ; le stamp de version est possédé par la release.
- **D-8b — Round-trip piloté par l'UI, une seule facture.** Le test suit un parcours utilisateur réel de bout en bout sur **la même facture** (pas des factures séparées par étape comme aujourd'hui). Il s'appuie sur le **seed lazy** des 3 niveaux par défaut (aucune création de niveaux requise ; une étape « ouvrir /settings/dunning et vérifier les 3 niveaux » suffit pour le volet config). Envoi unitaire + lot via l'UI, capture vérifiée via MockMailer.
- **D-8c — Mutualiser `fetchSentEmails`.** Le helper de capture d'e-mails est aujourd'hui **dupliqué mot pour mot** dans **DEUX** specs (`reminders.spec.ts:44-51` ET `invoice-send-email.spec.ts:49-56`). Le promouvoir dans `frontend/tests/e2e/helpers/` et re-pointer **les deux** specs dessus (DRY complet, cohérent A4 — ne pas laisser un duplicata nommé). Le helper local `pauseInvoiceViaApi` reste local (le round-trip exerce la suspension **par l'UI**, pas par API — cf. AC8).
- **D-8d — A4 mécanique.** Remplacer les 2 helpers locaux de `credit-notes.spec.ts` (`createContactViaApi`, `createAndValidateInvoiceViaApi`) par les fixtures partagées ; garder `login`/`uniq` locaux ; aucune assertion à ajuster (aucun montant testé — divergence adresse structurée / HT 1000→900 neutre).

### Hors scope (garde-fous)

- **Manuels DE/IT/EN** : vides en v0.1 (traductions v0.2+). 21-8 ne documente qu'en **FR** (cf. `CLAUDE.md` — DE/IT/EN = release majeure).
- **Bump des macros version** → release Epic 21 (D-8a). Ne pas les toucher ici.
- **Website / brochure marketing** : la synchro `website/` + brochure relève du **doc-sync de release** (`CLAUDE.md` liste de contrôle pré-release). 21-8 se limite aux **manuels admin/user** + CHANGELOG. Ne pas modifier `website/` ici.
- **Captures d'écran réelles** : le macro `\keshscreenshot` retombe sur `_placeholder.png` (aucune capture réelle n'existe). 21-8 pose les appels `\keshscreenshot{...}{légende}  % TODO capture` (patron existant), **sans** produire les PNG (chantier capture séparé).
- **Aucune nouvelle logique** : pas d'endpoint, pas de migration, pas de changement de comportement. Si un écart entre la doc et le comportement réel est découvert, c'est un **bug à tracer** (issue GitHub), pas à « corriger dans la doc ».
- **Promotion de `pauseInvoiceViaApi` en fixture partagée** → hors scope (le round-trip exerce la suspension **par l'UI**, `dunning-pause-*` ; sa mutualisation viendra si un futur spec la réutilise en API).
- **Étape « rappel manuel » dans le round-trip** → hors scope (déjà couverte par `reminders.spec.ts` ; le round-trip AC8 chaîne envoi **e-mail** unitaire+lot → historique → suspension → balance âgée, PAS le canal manuel). Le helper local `recordManualReminderViaApi` (`invoices.spec.ts:247`) reste donc local et n'est pas touché.

## Acceptance Criteria

### A. Manuel administrateur FR (`docs/manual/fr/admin-manual.tex`)

1. **Nouvelle sous-section « Rappels débiteurs (réglages) »** rattachée à la zone facturation/e-mail (après `\subsection{Modèles d'e-mail}` `:1154-1175`, patron structurel `\subsection{Envoi de factures par e-mail}` `:1095`). Contenu (prose FR, encadrés `keshnote`/`keshtip`/`keshwarning`) :
   - **Niveaux de rappel** : délai (jours après échéance) + frais par niveau (bornés 0–10'000.–), 3 niveaux par défaut seedés (1er rappel / 2e rappel / dernier avant poursuite), page **Réglages → Rappels débiteurs** (`/settings/dunning`).
   - **Période de grâce** (jours) et **échéancier prévisionnel** affiché.
   - **Modèles d'e-mail par niveau** : sélecteur de niveau, cascade (niveau N → générique 0 → défaut Rust), 4 langues.
   - **`keshwarning` sur les frais** : les **frais de rappel exigent une base contractuelle (CGV)** ; ils sont **affichés mais non comptabilisés** en v1 (produit accessoire) et **hors QR-facture** (le montant QR reste le TTC de la facture, pas les frais).
2. **Extension de la section « Conformité légale suisse »** (`:1836`, sous `\subsection{...Art. 958f...}` `:1852`) : un paragraphe **rétention des rappels** — chaque rappel envoyé est historisé (`invoice_reminders`, copie du texte + frais snapshotés = **preuve de recouvrement**), le destinataire `sent_to` est conservé **10 ans** (CO 958f), inclus dans l'**export de souveraineté** et la **sauvegarde admin** (D26, livré 21-3). Note LPD : donnée personnelle conservée pour obligation légale.

### B. Manuel utilisateur FR (`docs/manual/fr/user-manual.tex`)

3. **Nouvelle sous-section « Relancer les débiteurs »** dans `\section{Facturation QR Bill}` (`:520`), après `\subsection{Échéancier des factures}` (`:658`). Contenu (prose FR + `\keshscreenshot` en tête avec `% TODO capture`, patron `\subsection{Envoyer une facture par e-mail}` `:592`) :
   - **Écran Rappels** (`/invoices/reminders`) : liste des factures à rappeler **groupées par débiteur**, prochain niveau, badge « sans e-mail », état « dernier niveau atteint ».
   - **Envoi unitaire** (aperçu éditable, langue/ton du niveau, PDF joint), **envoi par lot** (jusqu'à 20, compte-rendu succès/échecs), **rappel manuel** (courrier/recommandé hors Kesh, saut de niveau autorisé).
   - **`keshtip` recommandé/mise en demeure** : pour le dernier niveau, privilégier l'envoi **recommandé** (traçabilité de la mise en demeure) — enregistrer le rappel papier via « rappel manuel ».
   - **Suspension/reprise** des rappels d'une facture (motif optionnel), badge « Suspendu », **invariant** : une facture suspendue **reste dans l'échéancier et la balance âgée** (elle sort seulement de la liste « à rappeler »).
   - **Historique des rappels** sur la fiche facture (date, niveau, canal, frais, destinataire ; rappel annulé barré).
4. **Correction de la phrase obsolète** `user-manual.tex:670` (« Cliquer sur une facture en retard permet de générer un rappel (template configurable). ») — la remplacer par un renvoi (`\ref`/`\nameref`) vers la nouvelle sous-section « Relancer les débiteurs » (le flux réel passe par l'écran Rappels, pas par un clic sur la ligne échéancier).
5. **Nouvelle sous-section « Balance âgée des créances »** dans `\section{Rapports comptables}` (`:973`, après `\subsection{Balance des comptes}` `:1004`) : onglet **Rapports → Balance âgée**, répartition de l'encours débiteur **TTC** par client et tranche d'ancienneté (**Non échu | 1-30 | 31-60 | 61-90 | 90+** jours), arrêté à ce jour, **total général** réconciliant avec le compte débiteurs, **export CSV** (réservé comptable/admin), lien vers les factures du client. `keshnote` : les factures suspendues **y restent comptées**.

### C. Régénération & commit des PDF

6. **Régénérer les 3 PDF FR** via `make fr` dans `docs/manual/` (xelatex ×2, cf. `Makefile`) et **commiter** `docs/manual/fr/{admin-manual.pdf, user-manual.pdf, marketing-brochure.pdf}` (convention projet — PDF versionnés, PR #102). **Ne PAS** modifier les macros version (D-8a). Si la chaîne LaTeX (`xelatex`) est absente de l'environnement, **HALT** et le signaler (la régénération PDF est un livrable, pas optionnelle) — cf. Dev Notes.

### D. E2E round-trip Playwright

7. **Promouvoir `fetchSentEmails`** (D-8c) **dans `frontend/tests/e2e/helpers/test-state.ts`** (là où vivent les autres helpers de test globaux `seedTestState`/`authedApiContext` — tranché validate P2, PAS un nouveau `mailer.ts`) : `export async function fetchSentEmails(page): Promise<Array<Record<string, unknown>>>` (corps identique à `reminders.spec.ts:44-51`, `GET ${BACKEND_URL}/api/v1/_test/sent-emails`). Re-pointer **les DEUX** specs sur l'import partagé et supprimer leur copie locale : `reminders.spec.ts:44-51` **ET** `invoice-send-email.spec.ts:49-56` (les 2 corps sont identiques — vérifié validate P1).
8. **Nouveau spec `frontend/tests/e2e/dunning-roundtrip.spec.ts`** — un test unique chaînant **une seule facture** (D-8b), piloté par l'UI, avec `seedTestState('with-company')` + `login` local + fixtures partagées (`createContactWithAddressViaApi` avec e-mail, `createAndValidateInvoiceViaApi(..., overdueDate())`). Étapes vérifiées bout-en-bout :
   - **(config)** `/settings/dunning` affiche les **3 niveaux seedés** + grâce (le seed lazy suffit, D-8b).
   - **(facture échue)** créer contact **avec e-mail** + facture échue (`overdueDate()`), + `ensurePrimaryBankAccountViaApi` (PDF/QR).
   - **(liste)** `/invoices/reminders` : la facture apparaît dans le groupe du contact.
   - **(envoi unitaire)** envoi UI (aperçu → confirmer) → **e-mail capturé** (`fetchSentEmails`, +1, `to` = e-mail contact, pièce jointe `.pdf`).
   - **(envoi lot)** une **2e** facture échue du même/deux contacts → sélection + envoi lot UI → rapport `{ accepted }` + **e-mail capturé** (+1).
   - **(historique)** `/invoices/{id}` de la 1re facture : la section **historique** montre le rappel envoyé (canal e-mail).
   - **(suspension)** toggle **Suspendre** (modale + **motif** saisi) sur la fiche → badge « Suspendu » ; **asserter que le motif est bien persisté** (`invoice-paused-badge` `title`/`aria-label` contient le motif — seule surface d'affichage du motif, `DunningPausedBadge.svelte:26`) ; **la facture reste dans la balance âgée** (étape suivante).
   - **(balance âgée)** `/reports?tab=aged-receivables` → Générer → la ligne du contact figure dans le tableau (facture suspendue **incluse**, D10).
   - **data-testid existants réutilisés** (aucun nouveau côté prod) : `dunning-level-row`, `reminder-send-open`/`reminder-send-confirm`, `reminder-batch-*`, `reminder-history`, `dunning-pause-button`/`dunning-pause-confirm`/`invoice-paused-badge`, `aged-report-generate`/`aged-receivables-row`.

### E. A4 — refactor `credit-notes.spec.ts`

9. **Migration mécanique** (D-8d) de `frontend/tests/e2e/credit-notes.spec.ts` :
   - Ajouter `import { createContactWithAddressViaApi, createAndValidateInvoiceViaApi } from './helpers/api-fixtures';`.
   - **Supprimer** les helpers locaux `createContactViaApi` (`:37-58`) et `createAndValidateInvoiceViaApi` (`:60-84`).
   - Remplacer les 2 appels `createContactViaApi(page, uniq(...))` (`:89`, `:127`) par `createContactWithAddressViaApi(page, uniq(...))`.
   - Les appels `createAndValidateInvoiceViaApi(page, contactId)` (`:90`, `:128`) restent inchangés (signature 2-params compatible).
   - **Garder** `login` (`:25`) et `uniq` (`:33`) locaux. **Aucune** assertion modifiée (les tests vérifient statut `cancelled`, préfixe `AV-`, en-tête PDF `%PDF-1.`, présence en liste — pas de montant).

### F. Documentation projet & CHANGELOG

10. **CHANGELOG** (`[Non publié]`) : ajouter une entrée `Modifié (technique)` ou `Ajouté` mentionnant la documentation du cycle de rappels + balance âgée dans les manuels admin/user (les entrées fonctionnelles Epic 21 existent déjà). **README** : Epic 21 déjà 🚧 — aucun changement de statut d'epic ici (le passage ✅ Done se fera à la clôture/rétro). **Website / brochure / macros version** → doc-sync de release (hors 21-8).

### G. Gate & documentation

11. **Gate local** (Test Locally First) :
    ```sh
    # Pré-vol (AVANT de commencer T1) — la chaîne LaTeX est requise pour T3 (AC 6).
    which xelatex >/dev/null || { echo "xelatex manquant (apt install texlive-xetex)"; exit 1; }
    # LaTeX — régénération des manuels
    cd docs/manual && make fr        # 3 PDF régénérés (xelatex ×2)
    # Frontend — les 2 specs touchés (credit-notes + round-trip) + reminders (re-pointé fetchSentEmails)
    cd frontend && npm run check && npm run lint-i18n-ownership && npm run test:unit && npm run build
    cd frontend && npm run test:e2e   # dunning-roundtrip.spec + credit-notes.spec + reminders.spec + invoice-send-email.spec (MockMailer requis)
    # Backend — aucun code Rust ni FTL touché : les 4 checks CLAUDE.md restent
    # exécutés par procédure (21-8 touche du TS E2E, pas « doc-only »), mais sont
    # des no-op cache-hit (0 delta Rust). Pas de gate workspace complet requis.
    cargo fmt --all -- --check
    cargo build --workspace --all-targets
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace   # (ou -p sur crates touchés — ici aucun ; no-op)
    ```
    ⚠️ E2E : backend `KESH_TEST_MODE` + **SMTP factice** (MockMailer) + `kesh_e2e` migré + `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64` + `KESH_COOKIE_SECURE=false` ; `npm run build` avant chaque run ; jamais de pipe sur le runner ; `cd frontend` explicite (cwd errant post-tâche de fond, leçon 21-6b). Le round-trip et `reminders.spec` **exigent le MockMailer** (`GET /_test/sent-emails`).
12. **Doc-only pour le backend** : aucun `.rs`, aucun `.ftl`, aucune migration → pas de gate workspace complet, pas de bump `min_required`. Le seul livrable « binaire » est les 3 PDF (versionnés).

## Tasks / Subtasks

- [ ] **T1 — Manuel admin : rappels (réglages) + LPD/rétention** (AC: 1, 2) — sous-section réglages (niveaux/grâce/modèles/CGV-frais-hors-QR) + paragraphe rétention `sent_to` CO 958f dans la section conformité.
- [ ] **T2 — Manuel user : relancer les débiteurs + balance âgée** (AC: 3, 4, 5) — sous-section cycle de relance (écran Rappels, unitaire/lot/manuel, recommandé/mise en demeure, suspension, historique) + correction phrase `:670` + sous-section balance âgée.
- [ ] **T3 — Régénérer & commiter les 3 PDF** (AC: 6) — `make fr`, commit des `.pdf` (macros version inchangées).
- [ ] **T4 — Promouvoir `fetchSentEmails`** (AC: 7) — helper partagé + re-pointer `reminders.spec.ts`.
- [ ] **T5 — E2E round-trip** (AC: 8) — `dunning-roundtrip.spec.ts`, une facture, cycle complet piloté UI, MockMailer.
- [ ] **T6 — A4 refactor `credit-notes.spec.ts`** (AC: 9) — migration mécanique vers fixtures partagées.
- [ ] **T7 — CHANGELOG + gate** (AC: 10, 11, 12).

## Dev Notes

### Pièges, par ordre de coût

1. **Chaîne LaTeX requise pour la régénération PDF (AC 6).** `make fr` utilise `xelatex` (pas latexmk). Si `xelatex` est absent de l'environnement de dev, la régénération échoue → **vérifier `which xelatex` d'abord** ; si absent, HALT et signaler (les PDF sont un livrable versionné, cf. PR #102). Ne PAS commiter un `.tex` sans son `.pdf` régénéré.
2. **Ne PAS toucher les macros version (D-8a).** `\keshVersion{0.6.0}` / `\keshReleaseDate{2026-07-11}` / `\keshTargetRelease{v0.6}` (`kesh-style.sty:64-66`) restent inchangées — c'est la release Epic 21 qui les bumpe (gate 4-bis). Régénérer avec les macros actuelles.
3. **Doc = reflet du comportement réel, pas aspirationnel.** La phrase `user-manual.tex:670` est un exemple de doc obsolète/aspirationnelle (« cliquer sur une facture en retard génère un rappel ») — le flux réel passe par l'écran Rappels. Vérifier chaque affirmation contre le comportement livré (21-3..21-7) ; un écart = bug à tracer, pas à documenter comme si c'était vrai.
4. **Round-trip : seed lazy des niveaux (D-8b).** Ne PAS créer les niveaux — ils sont seedés au 1er GET de `/settings/dunning` (3 niveaux + grâce 5). Le round-trip s'appuie dessus. `overdueDate()` (défaut 25 j) = éligible niveau 1 (seuil `today - 15j`).
5. **MockMailer obligatoire.** Le round-trip et `reminders.spec` échouent franchement si le backend est lancé sans SMTP factice (`GET /_test/sent-emails` 404). Recette : `docs/testing.md:149-160`.
6. **A4 : divergences neutres.** Le contact partagé est une `Personne` avec **adresse structurée** (#213) + `firstName/lastName` (vs texte libre local) ; la facture partagée fait **900.– HT** (vs 1000.– local). Les tests credit-notes n'assertent **aucun montant** → migration mécanique sûre. Garder `login`/`uniq` locaux (non partagés par convention).
7. **`bmad-code-review` sur une story doc+test.** La revue portera surtout sur : exactitude de la doc vs comportement réel, complétude du round-trip (chaîne réellement une seule facture ?), migration A4 sans régression de `credit-notes.spec`. Pas de logique métier à auditer.

### Contrats ground-truth (à ne pas re-deviner)

**Manuels** (agent Explore) :
| Élément | Emplacement |
|---|---|
| Ancrage admin rappels | `admin-manual.tex:1095` (§e-mail) / `:1154` (§modèles) |
| Ancrage admin LPD/958f | `admin-manual.tex:1836` (§conformité) / `:1852` (§958f) |
| Ancrage user relances | `user-manual.tex:520` (§Facturation) / `:658` (§Échéancier) |
| Phrase obsolète à corriger | `user-manual.tex:670` |
| Ancrage user balance âgée | `user-manual.tex:973` (§Rapports) / `:1004` (§Balance des comptes) |
| Macros version (NE PAS toucher) | `kesh-style.sty:64-66` |
| Encadrés | `kesh-style.sty:249` keshnote / `:269` keshtip / `:289` keshwarning / `:309` keshdanger |
| `\keshscreenshot` (par-tex, placeholder) | `user-manual.tex:15-23` |
| Build | `docs/manual/Makefile` : `make fr` (xelatex ×2), PDF commités |
| Précédent doc feature | Epic 20 e-mail : `admin-manual.tex:1095-1175`, `user-manual.tex:592-656` |
| Règle manuels | `CLAUDE.md:390-402` (gate 4 = contenu+régén ; gate 4-bis = release) |

**E2E** (agent Explore) :
| Élément | Emplacement |
|---|---|
| `fetchSentEmails` (à promouvoir) | `reminders.spec.ts:44-51` |
| `BACKEND_URL` | `reminders.spec.ts:22` (`KESH_BACKEND_URL ?? 'http://127.0.0.1'`) |
| Endpoint capture | `GET /api/v1/_test/sent-emails` (non authentifié, `docs/testing.md:149`) |
| Fixtures partagées | `api-fixtures.ts` : `overdueDate:18`, `createContactWithAddressViaApi:30`, `ensurePrimaryBankAccountViaApi:72`, `createAndValidateInvoiceViaApi:100` |
| Seed / auth | `test-state.ts` : `seedTestState:74`, `authedApiContext:169`, `clearAuthStorage:200` (pas de `login` partagé) |
| Config niveaux (seed lazy) | `/settings/dunning` ; `dunning.spec.ts:8,31` (3 niveaux au 1er GET) |
| testids envoi/lot | `reminder-send-open`/`-confirm`, `reminder-batch-checkbox`/`-send`/`-report` (`reminders.spec.ts`) |
| testids historique/suspension | `reminder-history`/`-row`, `dunning-pause-button`/`-confirm`, `invoice-paused-badge` (`invoices.spec.ts:245+`) |
| testids balance âgée | `aged-report-generate`, `aged-receivables-row`/`-total` (`reports.spec.ts:273+`) |
| Recette backend E2E | `docs/testing.md:149-160` (MockMailer + kesh_e2e + overrides) |

**A4** (agent Explore) :
| Élément | Emplacement |
|---|---|
| Helpers locaux à supprimer | `credit-notes.spec.ts:37-58` (`createContactViaApi`), `:60-84` (`createAndValidateInvoiceViaApi`) |
| Call-sites à renommer | `:89`, `:127` (contact) ; `:90`, `:128` (facture — inchangés) |
| À garder local | `login:25`, `uniq:33` |
| Assertions (aucune de montant) | statut `cancelled`, préfixe `AV-`, PDF `%PDF-1.` (`:115-119`), présence liste |
| Historique dette | rétro Epic 20 `epic-20-retro-2026-07-11.md:53,64` (item A4) |

### Project Structure Notes

**Nouveaux fichiers** :
- `frontend/tests/e2e/dunning-roundtrip.spec.ts`
- (pas de nouveau fichier helper : `fetchSentEmails` **ajouté à `helpers/test-state.ts`** existant, tranché validate P2)

**Modifiés** :
- `docs/manual/fr/admin-manual.tex` + `docs/manual/fr/user-manual.tex`
- `docs/manual/fr/{admin-manual,user-manual,marketing-brochure}.pdf` (régénérés)
- `frontend/tests/e2e/helpers/test-state.ts` (+`fetchSentEmails` exporté, D-8c)
- `frontend/tests/e2e/reminders.spec.ts` + `frontend/tests/e2e/invoice-send-email.spec.ts` (import `fetchSentEmails` partagé, D-8c)
- `frontend/tests/e2e/credit-notes.spec.ts` (A4)
- `CHANGELOG.md`

**Décompte** : ~2 surfaces (docs/manual, frontend/tests/e2e). Doc + test uniquement. Aucun code prod, aucun backend, aucune migration, aucun nouveau `data-testid` prod.

### References

- [Source: `epic-21-echeances-relances.md` — story 21-8 (ligne 106), item 26 (export/backup LPD 958f), item 27/A4, L21-8]
- [Source: rétro Epic 20 `epic-20-retro-2026-07-11.md:53,64` — dette A4 planifiée]
- [Source: cartographie ground-truth 3 agents Explore 2026-07-20 — manuels LaTeX, couverture E2E, refactor A4]
- [Source: `CLAUDE.md` §"Synchroniser TOUTES les docs" (gate 4 / 4-bis manuels), `#Test Locally First`, `#Issue Tracking Rule` ; `docs/testing.md` (recette MockMailer) ; `feedback_release_doc_sync_version_macro`]
- [Source: stories 21-3..21-7 (feature rappels + balance âgée documentée)]

## Change Log — validate

### Pass 1 (Sonnet ×2 : véracité citations + cohérence/complétude, 2026-07-20) — 2 MEDIUM + 2 LOW → patchés

Auteur spec : Opus. Panel orthogonal Sonnet. **~35 citations file:line vérifiées ground-truth — TOUTES exactes** (ancrages manuels LaTeX, macros, testids E2E, signatures fixtures, endpoints, SQL éligibilité/balance âgée, export/backup, historique git des macros). 0 CRITICAL/HIGH.

- **M1 (MEDIUM) — DRY partiel : `fetchSentEmails` dupliqué dans DEUX specs.** Corps identique mot pour mot dans `reminders.spec.ts:44-51` **et** `invoice-send-email.spec.ts:49-56` ; la spec ne migrait que le 1er. **Patch** : AC7/T4 + D-8c + File List re-pointent **les deux** specs (sinon la justification DRY est bancale).
- **M2 (MEDIUM) — contradiction Hors-scope vs AC8.** Le Hors-scope affirmait que le round-trip exerce `recordManualReminderViaApi` par l'UI, or AC8 n'a **aucune** étape rappel manuel (canal e-mail seulement). **Patch** : phrase corrigée — `pauseInvoiceViaApi` (suspension UI) reste local ; l'étape rappel manuel est explicitement **hors round-trip** (déjà couverte par `reminders.spec.ts`).
- **L1 (LOW) — gate AC11 trop étroit.** N'exécutait que `cargo fmt` ; procédure `CLAUDE.md` = 4 checks backend (21-8 touche du TS, pas « doc-only »). **Patch** : les 4 checks explicités (no-op cache-hit, 0 delta Rust).
- **L2 (LOW) — numérotation obsolète dans l'epic doc.** `epic-21-echeances-relances.md:83` disait « (dans 21-9…) » alors que la ligne 106 assigne A4 à 21-8 (21-9 n'existe pas). **Patch** : ligne 83 corrigée dans l'epic doc.

**Vérifications positives (Sonnet)** : complétude doc vs planning ligne 106 (tous sujets couverts) ; faisabilité round-trip prouvée par le SQL d'éligibilité (`dunning_eligibility.rs:119-133` — après envoi unitaire niveau 1, la facture sort de la liste → 2e facture bien nécessaire pour le lot) ; D10 (suspendue restant dans la balance âgée, `aged_receivables.rs:15`) ; D-8a (macros non bumpées = patron projet constant depuis PR #102, ex. Epic 20 `57fb87f5`→`8985bb33`) ; item 26 export/backup déjà livré (`global.rs:261-273`) ; A4 sans assertion de montant. **Aucun défaut de fond.**

### Pass 2 (Haiku ×2, contexte frais, 2026-07-20) — 0 CRITICAL/HIGH → **CONVERGÉ**

Panel Haiku orthogonal aux patches Opus/Sonnet. Les 4 correctifs P1 (M1/M2/L1/L2) re-vérifiés **présents et cohérents** ; ~45 citations file:line re-spot-checkées, **toutes exactes**.

- **Ground-truth (Haiku)** : **SPEC VALIDE, 0 finding**. Tous les ancrages LaTeX, macros, testids, fixtures, SQL, export/backup confirmés exacts ; patches P1 bien intégrés.
- **Cohérence (Haiku)** : 0 CRITICAL/HIGH, 3 clarifications de design (« affinage, non-bloquant, production-ready ») → **patchées** (reclassées LOW — pures précisions de spec, zéro impact logique) :
  - AC 7 : localisation `fetchSentEmails` tranchée → **`helpers/test-state.ts`** (pas de nouveau `mailer.ts`) ; Project Structure Notes + File List alignés.
  - AC 8 : ajout d'une **assertion de persistance du motif** de suspension (badge `title`/`aria-label`, seule surface d'affichage).
  - AC 6/Gate : ajout d'un **pré-vol `which xelatex`** avant T1 (évite de découvrir l'absence de la chaîne LaTeX après T1-T5).

### Trend & décision — validate

**Pass 1 (Sonnet ×2) : 2 MEDIUM + 2 LOW → Pass 2 (Haiku ×2) : 0 CRITICAL/HIGH (3 clarifications de design patchées, reclassées LOW).** Critère d'arrêt atteint (0 > LOW substantiel), budget 2/8. Rotation orthogonale Sonnet→Haiku, tous orthogonaux à l'auteur Opus. ~45 citations re-vérifiées ground-truth sur les 2 passes, aucune fausse. Les MEDIUM de fond (DRY partiel, contradiction Hors-scope) traités en P1 ; P2 n'a trouvé que des précisions. **Spec scellée, prête pour `bmad-dev-story 21-8`.**

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

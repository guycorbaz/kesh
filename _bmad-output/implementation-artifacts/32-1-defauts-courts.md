# Story 32.1 : défauts courts — tests et commentaires (32-1a), écran de la sauvegarde automatique (32-1b), procédure de répétition (32-1c) (#716, #711, #714)

Status: ready-for-dev

*Fiche rédigée le 2026-10-24, en autonomie (Opus 5.5), worktree `kesh-32-1`, branche `story/32-1` sur
`chore/kickoff-epic-32` (`6f0cee00`). Une fiche, trois clés (C-32-0-3, C-32-1-1) : `32-1a-defauts-tests-commentaires`,
`32-1b-defauts-sauvegarde-automatique`, `32-1c-defauts-procedure-repetition`. Registre :
`epic-32-choix-autonomes-321.md` (C-32-1-1 à C-32-1-28 ; C-32-1-20 à 28 de la validation P1). Clé courte : `321`. Action A8 de la rétrospective de l'Epic 31.*

## Story

En tant que Guy, qui tient sa comptabilité réelle dans Kesh, je veux que les constats LOW laissés par les revues de la
31-1c, de la 31-4 et de la 31-5 soient **réellement corrigés** quand ils sont mécaniques, et **sortis dans une issue
plus étroite** quand ils demandent une conception ou un geste à trancher, afin que les tests prouvent ce qu'ils
annoncent, que l'écran de la sauvegarde automatique ne perde ni une saisie ni un état, et que la procédure de
répétition d'une clôture dise vrai.

## Origine

Fichier d'epic, `_bmad-output/planning-artifacts/epic-32-serveur-mcp.md` § *32-1* : solder les LOW mécaniques de
[#711], [#714] et [#716], **fermés par correctif, jamais pour le chiffre** ; ce qui n'est pas mécanique va dans une issue
plus étroite, nommée dans le rapport.

**Décompte** (lecture seule, 2026-10-24, au kickoff : `gh issue list --state open --limit 500 --json number --jq length`
rendait **142**, dont 51 bugs — chiffres du fichier d'epic, non recomptés par cette fiche). Effet attendu de la story,
si tout se passe comme écrit : **#716** fermée par la 32-1a ; **#711** fermée par la dernière fusionnée de la 32-1a et
de la 32-1b (C-32-1-27), E-1 et E-2 sortis dans **[#725]** (ouverte le 2026-10-24) ; **#714** fermée par la 32-1c, les
gestes à trancher sortis dans **[#726]** (ouverte le 2026-10-24). Bilan : **−3 + 2 = −1** issue ; les bugs : #711 et
#714 portent `bug`, #716 non ; #725 porte `bug` (`bug`, `P3`), #726 non (`documentation`, `P3`) — **−2 + 1 = −1** bug
(étiquettes lues par l'orchestrateur à l'ouverture, recoupées par la validation P1). Ce n'est pas un motif pour taire un
constat (*On trace toujours*, `CLAUDE.md` § *Priorités des défauts*).

## Constats établis au code (2026-10-24, sur `6f0cee00`)

Chaque constat a été relu **au site**, même LOW (`docs/agents-orchestration.md` § 3). Colonne *Clé* : où il se traite ;
*hors* : sorti en issue, ou écarté.

### #716 — huit constats de la revue de la 31-5

| Id | Établi au code | Clé |
|---|---|---|
| B1-commentaire | **Vrai.** Le commentaire `invoice_pdf_service.rs:355-361` dit qu'« une facture et son rappel rendus dans la même langue portent le même message », sans dire qu'une facture **figée** avant la 0.15.0, ou figée dans une langue que le contact a quittée depuis, garde son ancien préfixe quand son rappel, régénéré à chaque envoi, porte celui de la langue courante. Le manuel utilisateur (`user-manual.tex:3262`) et le CHANGELOG (l. 112) le disent ; le commentaire non. | 32-1a |
| B2 | **Vrai.** `catalogues_charges()` (`audit_log_e2e.rs:432-441`) recopie le chargement de `spawn_app_de` (l. 99-107). | 32-1a |
| B3 = E3 | **Vrai.** `i18n.format(&locale, "invoice-qr-message-prefix", None)` (`invoice_pdf_service.rs:240`) ; le chargeur rend la clé brute si elle manque partout (`kesh-i18n/src/loader.rs:109-125`) ; aucun test n'exige la clé. | 32-1a |
| E1 | **Vrai.** `designate_vat_accounts_in_tx` (`company_invoice_settings.rs:1314-1334`, `COALESCE` l. 1321-1323) ; aucun appel hors de `kesh-seed/src/lib.rs:408`, aucun test dans `crates/kesh-db/tests/company_invoice_settings_repository.rs` (les quatre `vat_role_accounts_*`, l. 1687-1770, ne testent que la lecture). | 32-1a |
| E2 | **Vrai.** `latestLaterClosed` (`frontend/src/routes/(app)/settings/fiscal-years/+page.svelte:131-141`, filtre `other.status === 'Closed'` l. 134) ; les deux tests de la garde LIFO (`fiscal-years-page.test.ts:113`, `:134`) n'ont que des exercices clos. | 32-1a |
| E5 | **Vrai.** `construire_requete` (`audit_log.rs:171-217`) évalue `dateFrom` puis `dateTo` (l. 174-175) ; le tableau des cas (`audit_log_e2e.rs:367-394`, dix cas) ne combine pas deux dates invalides et n'exerce « code trop long » que sur `action` (`ACTION_MAX_LEN = 64`), jamais sur `entityType` (`ENTITY_TYPE_MAX_LEN = 32`, l. 42). | 32-1a |
| E6 | **Vrai.** `DATE_ISO = ^-\d{2}-\d{2}(\D\|$)` (`textes_coherents.rs:277-278`, employée l. 296) admet `1171-99-99` ; aucun test ne fixe la regex. | 32-1a |
| E7 | **Vrai.** Le doc-comment de `lock_vat_role_accounts_in_tx` (`company_invoice_settings.rs:1246-1254`) décrit l'ordre des trois verrous de TVA et nomme le rejeu ; il ne dit pas que la transaction **complète** de `final_body` n'est **pas croissante**. Séquence établie au code (C-32-1-23), comptes au plan PME (`crates/kesh-core/assets/charts/pme.json`) : les trois verrous de TVA (1171, 2200, 2206), puis les lectures verrouillantes d'`insert_with_defaults_in_tx` — ses trois rôles (l. 1074-1105 : `Receivable` 1100, `DefaultRevenue` 3000, `Payable` 2000), puis, par `chart_designated_accounts` (l. 443-493, appelé l. 1118) et `designated_account_id` (l. 501-521, `FOR UPDATE` l. 512), **jusqu'à quatre** comptes désignés par le plan, dans l'ordre du code : arrondi (6940), rabais (3800), frais bancaires (6900), pertes sur créances (3805) — dix au plus. | 32-1a |

### #711 — dix constats de la revue de la 31-1c (douze lignes : E-9 en trois)

| Id | Établi au code | Clé |
|---|---|---|
| E-1 | **Vrai, non mécanique.** `run_once` prend `backup_lock` en `try_lock_owned` (`auto_backup.rs:669`) et le tient jusqu'à la fin ; export et import l'attendent en `lock_owned().await` (`routes/admin.rs:77`, `:213`) ; `is_overdue` est faux sans `last_success_at` (`auto_backup.rs:221-226`). Un délai suppose de décider ce qu'il fait d'une écriture en cours. | **hors** — [#725] (C-32-1-14) |
| E-2 | **Vrai, non mécanique.** `record_success` (`auto_backup.rs:713`) en `Err` remonte par `?` : archive écrite, `last_success_*` inchangé, rétention non appliquée, 500 au `POST …/run` (`routes/backup_schedule.rs`, `??`), reprise dans une heure (`DELAI_DE_REPRISE`, `last_attempt_at` déjà posé) et seconde archive. Chemin d'erreur à concevoir. | **hors** — [#725] (C-32-1-14) |
| E-6 | **Vrai.** `runNow` (`AutoBackupSection.svelte:139-153`) → `apply` (l. 63-69) réécrit `enabled`, `intervalHours`, `retentionCount`. `record_*` ne touche jamais `version` (`crates/kesh-db/src/repositories/backup_schedule.rs:9-15`). | 32-1b |
| E-7 | **Vrai ; site corrigé.** Le délai est celui de `runBackupNow` (`backup-schedule.api.ts:95-100`, `LONG_OPERATION_TIMEOUT_MS` = 60 min) — **non** la l. 111 citée par l'issue, qui est `downloadServerBackup`. Un proxy coupe bien avant (502/504), et E-2 rend 500 archive écrite : dans tous ces cas, le `catch` de `runNow` (l. 146-151) dit « n'a pas pu être lancée » sans rien relire. | 32-1b |
| E-8 | **Vrai.** `refreshBackupScheduleState` (`backup-schedule.svelte.ts:30-36`) pose `null` sur tout échec ; aucune séquence ne protège `setBackupScheduleState` (l. 21-23) d'une réponse tardive. | 32-1b |
| E-9 (1) | **Vrai, et précisé.** `dossier_auto_inutilisable` (`admin_auto_backup_e2e.rs:684-695`) n'assert que `failed`. Les deux gardes du dossier (`auto_backup.rs:589`, création ; `:597`, `read_dir`) rendent `failed` ; toute création en échec fait aussi échouer `read_dir` : leur ordre est **équivalent** quant au statut (C-32-1-4). Ce qui est observable : le tour s'arrête **avant** la mesure d'espace (`free_bytes`) et avant toute écriture (`wrap_writer`). | 32-1a |
| E-9 (2) | **Vrai, non mécanique en test.** La propriété du garde de `full_export` (`routes/admin.rs:72-80`, garde déplacé dans `remplir`) ne s'observe qu'au départ du client pendant l'assemblage (C-32-1-5). | 32-1a (commentaire) |
| E-9 (3) | **Vrai.** `code()` (`auto_backup.rs:1121-1128`) n'écarte que les lignes `//` ; `argument_de` (`:1159-1179`) ne voit pas les chaînes ; `corps()` (`:1182-1188`) coupe au premier `\n}`. | 32-1a |
| L-B3 | **Vrai ; site corrigé.** `seuil_d_espace` (`auto_backup.rs:496-500` — non les l. 490-491 citées, qui sont la fin de `taille_des_documents`) : `max(dernière, documents) × 1,2 + 256 Mio` ; la base n'est mesurée qu'à travers la dernière sauvegarde. Le manuel (`admin-manual.tex:2349`, § *Espace disque requis*) ne le dit pas. | 32-1b |
| L-B5 | **Vrai.** `is_overdue` vrai pour une réussite future (`auto_backup.rs:221-226`, voulu) ; `since` (`BackupScheduleBanner.svelte:33`) affiche la date future. | 32-1b |
| L-B6 | **Vrai.** `formatBackupSize` en unités décimales (`backup-format.ts:14-28`) ; `exceedsImportLimit` en Mio (`backup-schedule.api.ts:129-131`) ; mention `AutoBackupSection.svelte:450-458`. | 32-1b |
| L-B7 | **Vrai.** `AppError::Validation("Réglage de sauvegarde hors domaine : …".into())` (`routes/backup_schedule.rs:133-137`). | 32-1b |

### #714 — constats de la revue de la 31-4 (manuel d'administration, `sec:repetition-cloture`, l. 1617-1933)

Les **deux choix de procédure** sont tranchés par Guy et versés en v0.15.0 (commentaire du 2026-10-24) : vérifié,
`admin-manual.tex:1791` (l'écart de pièces est un arrêt) et `:1906` (le `pull` en échec, le troisième listing). Ils ne
reviennent pas. Règle de partage des autres (C-32-1-15) : **la 32-1c corrige ce que la procédure dit** — une cause, une
sortie attendue, une source d'écriture, une limite —, **sans** ajouter, retirer ni changer **aucune commande** ni
**aucune consigne d'agir** ; le reste va dans [#726], pour Guy.

| Id (#714) | Établi au code | Clé |
|---|---|---|
| P3 E L-3 — `realpath -e` en échec | **En partie vrai.** « En cas d'arrêt », puce *Au contrôle~3* (`:1905`) : « laissez le dossier vide, et traitez la cause (un `/var/tmp` qui serait un lien) ». La cause la plus probable est une **barre finale** dans `COPIE` (`realpath` la retire, la comparaison échoue), que la puce ne nomme pas ; *Conventions* (`:1635`) dit `COPIE` « toujours écrit sans barre finale ». « Un reste qui **bloque** une reprise le même jour » est **faux** : la reprise se fait dans un nouveau dossier, suffixé (`:1635`) ; le reste est un dossier vide. | cause : 32-1c ; geste (retirer la barre et supprimer par la garde) : **hors** — [#726] |
| P2 B2-L4 = E-LOW-2 — inventaire du contrôle 4 muet | **Vrai.** Si un `curl -fsSL … -o` échoue (`:1704-1705`), le `grep` lit un fichier absent et la boucle n'affiche rien : « Attendu : rien » est satisfait à tort ; le contrôle 7 rattrape. | 32-1c (sortie attendue) |
| P2 E-LOW-7 — clients par clé d'API | **Vrai.** Étape 0 (`:1686`) : « ni saisie, ni import, ni autre utilisateur au travail » — une intégration par clé d'API qui écrit n'est pas nommée. « Refaire les exports en cas de doute » est une consigne neuve. | nommer : 32-1c ; refaire : **hors** — [#726] |
| P1 B-10 — gestionnaire de mots de passe, historique | **Vrai.** Rien n'avertit que le navigateur propose d'enregistrer, pour `127.0.0.1:18314`, les identifiants de la production saisis à la reconnexion (`:1788`), ni que l'historique des téléchargements garde les noms de fichiers. | **hors** — [#726] (consignes neuves) |
| P1 E LOW-2 — port pris entre le contrôle 8 et `up -d` | **Vrai.** Aucune branche ne dit ce que fait un `up -d` qui échoue (contrôle 9, `:1772`). | **hors** — [#726] (choix de branche) |
| P1 E LOW-3 — `mariadb:10.11` en dur dans la destruction | **Vrai.** `:1844` et `:1906` ; juste au tag 0.15.0 (`docker-compose.yml:4`), faux si un compose ultérieur change d'image. | **hors** — [#726] (consigne « remplacez ») |
| P1 E LOW-4 — secrets en arguments de `sed`, visibles de `ps` | **Vrai.** Contrôle 5, `:1714-1720`. | **hors** — [#726] (commande à changer) |
| P1 E LOW-5 — `export KESH_…_HOST_DIR=` non reconnu au contrôle 6 | **Vrai à la regex** (`:1726`), **sans cas réel** : le `.env` vient de `.env.example` du tag (`.env.example:235-266` : lignes commentées, aucun `export`), modifié par le seul `sed` du contrôle 5. | **hors** — [#726] (commande à changer) |
| P4 LOW-2 — branche « date inconnue » ; dossier sous un autre nom | **En partie vrai.** La branche « date inconnue » (`:1914`) est dans la même puce que le `ls -d` qu'elle suppose (premier geste de la puce *Reste d'une répétition antérieure*, `:1907`) : tenue. Le contrôle final (`:1880-1900`) ne regarde que `$COPIE` : un dossier d'une **autre** répétition, sans objet Docker, lui échappe — limite non écrite. | limite : 32-1c |
| P4 LOW-1, LOW-3 — « elle ne fait que lire la version » | **Vrai.** `:1625` : « elle ne fait que lire la version et télécharger deux exports » contredit la suite de la phrase, qui dit les écritures. Et « comme le fait la première ouverture de leur page » est inexact pour les **relances** : l'export appelle `company_dunning_settings::get_or_create_default` (`crates/kesh-api/src/exports/global.rs:313`, un `INSERT IGNORE` de la seule ligne), la page des relances appelle `ensure_seeded_in_tx` (`routes/company_dunning_settings.rs:56`), qui pose **en plus** les trois niveaux par défaut (`crates/kesh-db/src/repositories/company_dunning_settings.rs:168-200`). | 32-1c |
| P3 B3-2 — « Ou, avec sudo : » détaché | **Vrai.** `:1851-1852` : l'amorce précède le `\needspace{10\baselineskip}`, qui ne la retient donc pas avec son listing. | 32-1c |
| P2 B2-L5 — ⚠️ non imprimé | **Vrai, et plus large.** `grep -c "⚠"` (des **lignes**) : **16** dans `admin-manual.tex`, **9** dans `user-manual.tex`, 0 dans la brochure ; en **occurrences** (`grep -o "⚠" … \| wc -l`) : **17** et **9**, soit **26** (la l. 2923 d'`admin-manual.tex` en porte deux), toutes écrites `⚠️` (U+26A0 suivi du sélecteur U+FE0F : `grep -o $'\xe2\x9a\xa0\xef\xb8\x8f' docs/manual/fr/*.tex \| wc -l` rend 26), aucune dans un `lstlisting` ; `pdftotext … \| grep -c "⚠"` rend **0** pour les deux PDF (police TeX Gyre Heros, `kesh-style.sty:95`). | 32-1c, les deux manuels (C-32-1-16) |
| Fiche P2 A-LOW-4 — G15 et l'AC11 | **Vrai.** L'AC11 de la 31-4 (`31-4-repetition-a-blanc.md:537-543`) range G15 parmi « les gardes existantes qui relisent la sous-section » ; G15 (`configuration_transmise.rs:2732-2745`) lit `/var/lib/docker/volumes/<x>` et « volume `\texttt{<x>}` » — la sous-section écrit ses volumes en `\keshpath{…}` : G15 n'y lit rien. | 32-1c (annotation datée) |
| Fiche P4 LOW-4 — `test ! -e` au présent | **Vrai.** `31-4-repetition-a-blanc.md:504` (puce *nom faux*, AC7 (d)) décrit au présent un test que le manuel n'écrit plus ; la l. 433 est annotée (l. 435), la l. 504 non. | 32-1c (annotation datée) |

**Prémisses corrigées**, à signaler sur les issues (C-32-1-19) : #711 E-7 (site l. 95-100, non 111), L-B3 (l. 496-500,
non 490-491), E-9 (1) (distinction observable) ; #714 P3 E L-3 (« bloque une reprise » : faux) ; #716 E6 (l. 277-278 et
296) ; fichier d'epic, 32-1a : « M4 de la 31-5 tuée » — la M4 de la 31-5 est **équivalente** et le reste ; c'est la
mutation du `COALESCE` qui se tue (C-32-1-2).

---

## A — 32-1a : tests et commentaires (#716 entier ; E-9 de #711)

**Modules** : `kesh-api` (tests : `audit_log_e2e.rs`, `textes_coherents.rs`, `admin_auto_backup_e2e.rs` ; `src` : un
commentaire de `routes/invoice_pdf_service.rs` et son `mod tests`, le `mod tests` de `auto_backup.rs`), `kesh-db`
(doc-comment de `repositories/company_invoice_settings.rs`, `tests/company_invoice_settings_repository.rs`), frontend
(`settings/fiscal-years/fiscal-years-page.test.ts`) — **3**. 🗄️ (un test de dépôt `kesh-db`) : **gate complet même en
boucle de revue**. Aucune ligne servie ne change ; `kesh-i18n` et `kesh-seed` ne sont pas touchés (C-32-1-3, C-32-1-7).

### Critères d'acceptation

1. **AC-a1 — B1 : le commentaire dit le cas figé.** `invoice_pdf_service.rs:355-361` complété : une facture dont le PDF a
   été **figé** (avant la 0.15.0, ou dans une langue que le contact a quittée depuis) garde le préfixe de ce PDF, tandis
   que son **rappel**, produit à chaque envoi, porte celui de la langue courante du client ; le rapprochement n'en est pas
   touché (il lit la contenance du numéro). Commentaire seul ; aucun test.
2. **AC-a2 — B2 : un seul chargement des catalogues.** Dans `audit_log_e2e.rs`, `spawn_app_de` emploie
   `catalogues_charges()` (déplacée au-dessus si besoin) au lieu de sa copie ; `grep -c 'I18nBundle::load'
   crates/kesh-api/tests/audit_log_e2e.rs` rend **1** (aujourd'hui 2). Le binaire reste vert (`binary(audit_log_e2e)`).
3. **AC-a3 — B3 : la clé du préfixe de la QR est exigée.** Test unitaire neuf dans `invoice_pdf_service.rs`
   (`mod tests`, catalogues réels chargés comme par `reminder_fees_note_fits_its_width_in_all_four_locales`, l. 1005) :
   pour `FrCh`, `DeCh`, `ItCh`, `EnCh`, `bundle.format(&locale, "invoice-qr-message-prefix", None)` n'est **ni vide ni
   égal à la clé** ; pour `DeCh`, `ItCh`, `EnCh`, il **diffère** du rendu `FrCh`. **Rouge constaté** : la clé retirée des
   quatre catalogues (mutation **M-a4**, restaurée ensuite) fait rougir ce test ; retirée du seul `de-CH`, il rougit sur
   « diffère du fr-CH » (le chargeur se replie sur le français). Aucun repli Rust (C-32-1-3).
4. **AC-a4 — E1 : « jamais d'écrasement » et « idempotent » exercés, sur les trois colonnes** (C-32-1-21). Deux tests
   neufs dans `crates/kesh-db/tests/company_invoice_settings_repository.rs`, à côté des `vat_role_accounts_*` (même
   montage, `pme_company_with_chart`, l. 1635). Montage transactionnel, dit pour qu'aucun verrou ne s'attende lui-même :
   **(t1)** `pool.begin()`, `insert_with_defaults_in_tx`, `commit` ; **(u)** les `UPDATE` directs sur le **pool**, après
   ce `commit` ; **(t2)** `pool.begin()`, `lock_vat_role_accounts_in_tx`, `designate_vat_accounts_in_tx`, `commit` ;
   relecture des colonnes et de `version` sur le pool ; **(t3)** comme (t2), seconde fois.
   - **`designate_vat_never_overwrites_a_set_value`** — en (u), les **trois** colonnes posées à trois valeurs
     **légitimes** (du type que `VAT_SETTING_ACCOUNT_TYPES`, l. 1223-1227, exige, imputables) et **distinctes** des
     comptes des rôles, lues par numéro au plan PME : `default_vat_payable_account_id` ← 2000 (`Liability`),
     `default_vat_recoverable_account_id` ← 1170 (`Asset`), `default_vat_decompte_account_id` ← 2201 (`Liability`).
     Assertion de montage : les trois comptes sont `postable`, du bon type, et différents de ceux que rend
     `lock_vat_role_accounts_in_tx` (sans quoi l'égalité finale ne prouve rien). Après (t2) : les trois colonnes
     **restent** les valeurs posées.
   - **`designate_vat_fills_nulls_and_is_idempotent`** — sans (u) : après (t2), les trois colonnes égalent les comptes
     des rôles ; après (t3), les trois colonnes **et** `version` sont inchangées (la fonction ne touche pas `version`,
     doc-comment l. 1311-1312).
   **Mutations M-a1a, M-a1b, M-a1c** : `COALESCE(<colonne>, ?)` → `?` pour `default_vat_payable_account_id`,
   `default_vat_recoverable_account_id`, `default_vat_decompte_account_id` respectivement (l. 1321, 1322, 1323) — chacune
   rougit `designate_vat_never_overwrites_a_set_value` (sa colonne prend le compte du rôle). La M4 de la 31-5 reste
   **équivalente** et n'est pas visée (C-32-1-2).
5. **AC-a5 — E2 : un clos suivi d'un ouvert reste rouvrable.** Cas neuf dans `fiscal-years-page.test.ts`, bloc
   `garde LIFO client` : 2025 **clos**, 2026 **ouvert** ; `fiscal-year-reopen-1` **activé**, sans `title` sur le bouton
   ni sur son enveloppe. **Mutation M-a2** (la M9 de la revue de la 31-5) : retirer `other.status === 'Closed' &&` de
   `latestLaterClosed` (`+page.svelte:134`) → ce cas rouge.
6. **AC-a6 — E5 : ordre des refus et borne d'`entityType`.** Deux cas de plus au tableau de
   `les_parametres_invalides_sont_refuses_en_400` (`audit_log_e2e.rs:367`, `[Cas; 10]` → `[Cas; 12]`) :
   - `?dateFrom=pas-une-date&dateTo=pas-une-date` → `audit-log-query-error-date-invalid`, `field = dateFrom` (l'ordre
     d'évaluation est figé : `dateFrom` d'abord) ;
   - `?entityType=` suivi de **33** caractères → `audit-log-query-error-code-too-long`, `field = entityType`,
     `max = 32`.
   **Mutations** : **M-a5** — permuter les deux `parse_date` (l. 174-175) → le premier cas rouge ; **M-a6** — passer
   `ACTION_MAX_LEN` au lieu de `ENTITY_TYPE_MAX_LEN` (l. 188) → le second rouge (200 au lieu de 400).
7. **AC-a7 — E6 : la date ISO bornée, et la regex fixée par des cas.** `DATE_ISO` (`textes_coherents.rs:277-278`)
   devient `^-(0[1-9]|1[0-2])-(0[1-9]|[12]\d|3[01])(\D|$)`. Test neuf (`#[test]`, même fichier) sur `nombres_isoles` :
   **négatifs** (aucun nombre rendu) `"entre 1000-01-01 et 9999-12-31"`, `"au 2026-02-28."` ; **positifs** (le nombre
   est rendu) `"1171-99-99"`, `"1171-13-01"`, `"1171-00-10"`, `"1171-12-32"`. **Rouge constaté avant le correctif** :
   les quatre positifs rendent aujourd'hui une liste vide. Limite assumée, écrite au doc-comment : `2026-02-30` reste
   reconnue comme date (la regex ne connaît pas la longueur des mois). `les_comptes_cites_en_exemple_existent_dans_les_plans_livres`
   et `la_forme_libre_nnnn_nom_est_juste_partout` restent verts.
8. **AC-a8 — E7 : la séquence complète des verrous dite.** Doc-comment de `lock_vat_role_accounts_in_tx`
   (`company_invoice_settings.rs:1246-1254`) complété (C-32-1-23) : dans `final_body`, ces trois verrous précèdent les
   lectures verrouillantes d'`insert_with_defaults_in_tx` — ses trois rôles (`Receivable`, `DefaultRevenue`, `Payable`),
   puis les comptes que le plan désigne (`chart_designated_accounts` : arrondi, puis les natures d'écart soldé dans
   l'ordre de `WriteOffNature::ALL`, chacun seulement si le plan le désigne). Au plan PME, la séquence par numéro est
   1171, 2200, 2206, 1100, 3000, 2000, 6940, 3800, 6900, 3805 : elle **n'est pas croissante**, et le rejeu de
   `seed_demo` sur interblocage (`is_seed_retryable`) est le filet. Le doc-comment **n'énumère pas** comme exhaustive
   une liste qu'il ne tient pas : il renvoie aux deux fonctions qui la portent. Commentaire seul ; `kesh-seed` non
   touché (C-32-1-7).
9. **AC-a9 — E-9 (1) : le tour s'arrête à la garde du dossier.** `TestEnv` (`admin_auto_backup_e2e.rs:55-129`) compte
   ses appels à `free_bytes` (compteur `Arc<AtomicUsize>` lisible par le test, ou une variante `TestEnv::sonde()` qui le
   rend ; les appels existants de `large()` et `avec()` inchangés) ; `dossier_auto_inutilisable` assert, en plus de
   `failed`, **zéro** appel. Doc-comment du test : les deux gardes du dossier (création, lecture) rendent le même
   statut, et leur ordre est équivalent (C-32-1-4) ; `wrap_writer` **n'est pas compté** (C-32-1-25) : sur ce montage,
   même gardes neutralisées, `write_backup_file` échoue à créer `auto/<nom>.partial` sous un fichier (`ENOTDIR`,
   `routes/admin.rs:747`) **avant** d'appeler `remplir`, donc `wrap_writer` ; un compteur de `wrap_writer` serait à zéro
   avec ou sans le défaut. **Mutation M-a3** : neutraliser les deux `return Ecriture::Refusee(FailureStatus::Failed);`
   des gardes du dossier (`auto_backup.rs:591`, `:599`) → le tour passe `supprimer_les_partiels` et
   `taille_des_documents`, **mesure l'espace** (compteur à 1 : rouge), puis échoue à l'ouverture du `.partial` — le statut
   reste `failed`.
10. **AC-a10 — E-9 (2) : la garde lexicale de l'export, assumée.** Doc-comment de `gardes_possedes`
    (`auto_backup.rs:1190-1201`) : la propriété tenue par le déplacement du garde de `full_export` dans `remplir` (le
    verrou vit jusqu'à la fin de l'assemblage même si le client part) ne s'observe qu'au départ du client pendant
    l'assemblage ; aucun test comportemental ne la tient (il faudrait un crochet de synchronisation dans
    `build_keshbackup`) — angle mort assumé, la garde lexicale (M-c24) est le seul filet (C-32-1-5). Commentaire seul.
11. **AC-a11 — E-9 (3) : les helpers lexicaux lisent ce qu'ils disent** (C-32-1-20).
    - **Un seul balayeur**, dans `mod tests`, partagé par `code()` et `argument_de` : il parcourt le texte caractère par
      caractère et distingue cinq états — code, commentaire `//` (jusqu'à la fin de ligne), commentaire `/* … */`
      (imbriqué, avec profondeur), chaîne `"…"` (échappements `\"` et `\\`), chaîne brute `r"…"` / `r#"…"#` (sans
      échappement, fermée par `"` suivi d'autant de `#`) — et les littéraux de caractère **seulement** sous les formes
      `'x'` et `'\x'` (apostrophe, un caractère ou un échappement, apostrophe) : une apostrophe qui n'ouvre pas l'une
      de ces formes est une **durée de vie** (`&'static str`, présent dans `auto_backup.rs:258` et
      `routes/backup_schedule.rs:194`, lus par `gardes_possedes`) et reste du code. Un `/*` ou un `//` **dans une
      chaîne** n'ouvre aucun commentaire.
    - `code()` rend le code de production, commentaires (`//` et `/* … */`) **retirés par le balayeur**, et garde sa
      troncature au premier `#[cfg(test)]`. Assertions de montage, toutes **rouges avant** sauf mention :
      (1) `code("/* auto_backup::spawn(x) */ fn f() {}")` ne contient pas `auto_backup::spawn(` ;
      (2) `code("/* a /* b */ auto_backup::spawn(x) */ fn f() {}")` non plus ;
      (3) **tirée du réel** — `lib.rs:1186` porte `/*` **dans une chaîne** (`"KESH_TEST_MODE=true — /api/v1/_test/*
      exposé …"`), sans `*/` dans tout le fichier et sans `#[cfg(test)]` qui borne la lecture : un retrait de blocs qui
      ignore les chaînes ouvrirait là un commentaire jamais fermé et ferait disparaître la fin de `build_router` de la
      garde « zéro lancement dans `lib.rs` », **en vert**. Assertion : `code("let s = \"a /* b\"; auto_backup::spawn(x);
      fn g() {}")` contient `auto_backup::spawn(` (verte avant, **rouge sous un retrait naïf** — c'est la mutation
      **M-a7** : un `code()` qui retire `/*…*/` sans connaître les chaînes) ;
      (4) **sur le vrai fichier** — `code(include_str!("lib.rs"))` contient `.with_state(state)` (`lib.rs:1200`, dernière
      instruction de `build_router`, **après** la l. 1186) : verte avant, rouge sous M-a7 ;
      (5) une durée de vie n'ouvre rien : `code("fn f() -> &'static str { \"x\" } auto_backup::spawn(y)")` contient
      `auto_backup::spawn(` (verte avant ; elle tient le balayeur contre un `'` pris pour un littéral ouvrant) ;
    - `argument_de` apparie les parenthèses **dans l'état « code » du balayeur seulement** ; assertion de montage : sur le
      texte `f("(", x) y` (littéral Rust `"f(\"(\", x) y"`), `argument_de(texte, "f(")` rend `("(", x)` — **rouge avant**
      (aujourd'hui : panique, la parenthèse de la chaîne laissant la profondeur à 1 au bout du texte) ; et sur `f('(', x)`,
      il rend `('(', x)` ;
    - `corps()` garde sa règle ; son doc-comment dit qu'elle repose sur rustfmt (un item de tête se ferme par `}` en
      colonne 0) ; assertion de montage : `corps(&admin, "pub async fn full_import(")` ne contient pas `"\npub "`.
    `lancement_par_main_seulement` et `gardes_possedes` restent verts.
12. **AC-a12 — gates.** Gate **complet** (`scripts/test-fast.sh` sous verrou, base remise à zéro d'abord) ; les quatre
    gates frontend ; **E2E complet au dernier commit de code** (C-32-1-18), jugé fichier par fichier contre
    `docs/testing.md` § *Les échecs attendus*. Ne déclarer que ce qui a tourné.
13. **AC-a13 — mutations** (sous verrou, appliquées par Edit, restaurées par `git checkout -- f && touch f`, diff
    résiduel 0, un journal `321-mut-<id>.log` chacune dans `kesh-gate-logs/`) : M-a1a, M-a1b, M-a1c, M-a2 à M-a7 (neuf), chacune tuée par un test
    **nommé**. Une mutation qui survit est un défaut du test, non un constat à reporter.

### Tâches

- [ ] **T-a0** — Base `kesh_321`, `kesh_e2e_321`, port attribué par l'orchestrateur,
  `CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh-32-1/target` (`docs/agents-orchestration.md` § 4).
- [ ] **T-a1 (AC-a7)** — test de `nombres_isoles` écrit, **rouge constaté**, puis la regex ; `binary(textes_coherents)`.
- [ ] **T-a2 (AC-a3)** — test du préfixe ; M-a4 jouée (rouge), restaurée.
- [ ] **T-a3 (AC-a4)** — deux tests de dépôt ; M-a1a, M-a1b, M-a1c ; gate complet (kesh-db).
- [ ] **T-a4 (AC-a5)** — cas Vitest ; M-a2.
- [ ] **T-a5 (AC-a2, AC-a6)** — `audit_log_e2e.rs` ; M-a5, M-a6.
- [ ] **T-a6 (AC-a9, AC-a11)** — sonde, balayeur et helpers lexicaux ; assertions de montage (1), (2) et `argument_de` **rouges avant**, (3) à (5) vertes avant et (3), (4) **rouges sous M-a7** ; M-a3, M-a7.
- [ ] **T-a7 (AC-a1, AC-a8, AC-a10)** — trois commentaires.
- [ ] **T-a8 (AC-a12, AC-a13)** — gates complets, E2E complet ; Dev Agent Record chiffré.

---

## B — 32-1b : l'écran de la sauvegarde automatique (#711 : E-6, E-7, E-8, L-B3, L-B5, L-B6, L-B7)

**Modules** : frontend (`lib/features/admin-backup/`), `kesh-api` (`routes/backup_schedule.rs`, `auto_backup.rs` pour
L-B5 ; tests `i18n_e2e.rs`, `admin_auto_backup_e2e.rs`, `textes_coherents.rs`), `kesh-i18n` (catalogues, quatre locales),
manuel d'administration FR (`.tex` et PDF) — **4**. Pas de migration, pas de `kesh-db`.

### Critères d'acceptation

1. **AC-b1 — E-6 : une saisie non enregistrée survit à « Sauvegarder maintenant ».** `AutoBackupSection.svelte` :
   `applyState(value)` pose `schedule` et l'état partagé, et ne remet les trois champs du formulaire à `value` **que
   s'ils égalent encore le réglage lu** (`schedule` d'avant) ; `runNow` l'emploie ; `loadSchedule` et `save` gardent
   `apply` (C-32-1-8). Vitest neuf (bloc « Sauvegarder maintenant ») : la fréquence passée de 24 à 12 **sans
   enregistrer**, puis run réussi (`runBackupNow` rend `lastStatus: 'ok'`, `intervalHours: 24`) → le sélecteur montre
   **12**, l'état affiché est celui de la réponse (dernière tentative réussie), et « Enregistrer » envoie
   `intervalHours: 12` avec la `version` de la réponse. Et le cas non modifié : sans saisie, après un run dont la réponse
   porte `retentionCount: 9`, le champ montre 9. **Rouge constaté avant le correctif** (le premier cas montre 24).
   **Mutation M-b1** : `applyState` qui réécrit toujours le formulaire → rouge.
2. **AC-b2 — E-7 : une réponse qui manque relit l'état** (C-32-1-9, amendé par C-32-1-22). Dans `runNow`, l'échec se
   classe en trois cas, et le message « pas parvenue » est **réservé** à celui où le serveur n'a pas répondu :
   - **issue inconnue** — `isApiError(err)` et `code === 'TIMEOUT'`, `code === 'NETWORK_ERROR'` (tous deux `status: 0`,
     `api-client.ts:312-321`), `status === 502` ou `status === 504` : le prédicat **même** d'`issueInconnue`
     (`features/admin-restore/AdminRestorePanel.svelte:61-68`), à **extraire** dans un module partagé
     (`frontend/src/lib/shared/utils/`, nom au développement) et employé par les deux écrans (règle DRY ; aucun
     comportement de l'import ne change) → `runError` = `admin-backup-auto-run-unknown` (« La réponse du serveur n'est
     pas parvenue : la sauvegarde a peut-être été écrite. L'écran relit l'état et la liste ci-dessous : vérifiez-les. »,
     valeur fr-CH à reprendre telle quelle ou à affiner, quatre locales — elle ne dit **pas** que la relecture a réussi),
     puis relecture de l'état et de la liste ;
   - **autre réponse 5xx** (500 d'E-2 compris, 503) : le serveur a répondu, et l'archive a pu être écrite (E-2) →
     `runError` = le message du serveur (`errorText`, repli `admin-backup-auto-run-error` actuel), **et** relecture de
     l'état et de la liste ;
   - **4xx** (401, 403, 409 `BACKUP_IN_PROGRESS`, 423, 429…) : le serveur a refusé, rien n'a été écrit → message du
     serveur (`errorText`, comportement actuel), **aucune** relecture.
   **Relecture** : l'état par `fetchBackupSchedule` → `applyState` (non par `loadSchedule`, qui remet `conflict` et
   `saveError` et réécrit le formulaire) ; la liste par `loadBackups`. Un échec de la relecture de l'état pose
   `loadError` (même texte que `loadSchedule`, `AutoBackupSection.svelte:78-84` : alerte `auto-backup-load-error`
   au-dessus du panneau, le réglage affiché restant le précédent) ; un échec de la liste pose `listError` (déjà fait par
   `loadBackups`). Vitest neufs : `{ code: 'TIMEOUT', status: 0 }` → message neuf, `fetchBackupSchedule` appelé une fois
   de plus qu'au montage, `fetchServerBackups` aussi ; idem `{ status: 504 }` ; `{ status: 500, message: 'Erreur
   interne' }` → « Erreur interne », relecture faite ; `{ status: 403, message: 'Accès refusé' }` → « Accès refusé »,
   **aucune** relecture ; `TIMEOUT` puis `fetchBackupSchedule` rejeté → message neuf **et** `auto-backup-load-error`
   présent ; le test 409 existant (`AutoBackupSection.test.ts:210`) reste vert **et** assert qu'aucune relecture d'état
   n'a lieu ; les tests d'`AdminRestorePanel` restent verts (prédicat extrait). **Rouges constatés avant le correctif**
   (TIMEOUT, 504, 500). **Mutations** : **M-b2** — retirer la relecture → rouge ; **M-b8** — étendre le message neuf aux
   4xx (prédicat `status !== 409`) → le cas 403 rouge.
3. **AC-b3 — E-8 : un échec de sondage n'éteint plus le bandeau ; une réponse tardive est ignorée.**
   `backup-schedule.svelte.ts` : un échec de lecture **laisse** `_value` ; un compteur de séquence, incrémenté par
   `setBackupScheduleState` et au départ de chaque `refreshBackupScheduleState`, fait ignorer toute réponse dont la
   séquence n'est plus la dernière (C-32-1-10). Doc-comments du module et de `BackupScheduleBanner.svelte` (l. 10-12)
   mis d'accord. Vitest neufs (`BackupScheduleBanner.test.ts`) : (i) état `failed` affiché, sondage suivant en échec →
   bandeau **toujours** `failed` ; (ii) une lecture lancée, puis `setBackupScheduleState(ok)`, puis la lecture rend
   `failed` → **aucun** bandeau ; (iii) le test existant « lecture en échec : absent, sans erreur » (l. 106) reste vert.
   **Rouges constatés avant le correctif.** **Mutations** : **M-b3** — rétablir `_value = null` dans le `catch` →
   (i) rouge ; **M-b4** — ignorer la séquence → (ii) rouge.
4. **AC-b4 — L-B5 : une réussite datée du futur se dit telle.**
   - `auto_backup.rs` : `pub fn is_success_in_future(schedule: &BackupSchedule, now: NaiveDateTime) -> bool`, vrai si
     la sauvegarde est **activée**, que `last_success_at` existe et que `passe()` l'écarte (l. 183) — la garde
     `enabled` calquée sur `is_overdue` (l. 221-226), pour que `lastSuccessInFuture` vrai implique `overdue` vrai et que
     l'API ne serve jamais deux drapeaux contradictoires (C-32-1-24) ; tests unitaires à côté de
     `is_overdue_dans_le_futur` (l. 950) : futur → vrai, passé → faux, absent → faux, égal à `now` → faux, **futur et
     désactivée → faux**.
   - `BackupScheduleResponse` (`routes/backup_schedule.rs:55-74`, `new` l. 76-95) : champ `last_success_in_future: bool`
     (`lastSuccessInFuture`), calculé par `new` ; `BackupSchedule` (frontend, `backup-schedule.api.ts:34-51`) le porte,
     **requis** (le serveur le sert toujours ; C-32-1-28). Fixtures typées à compléter de `lastSuccessInFuture: false`,
     sans quoi `npm run check` rougit : `backup-schedule.api.test.ts:44` (`schedule()`), `AutoBackupSection.test.ts:45`,
     `BackupScheduleBanner.test.ts:43` ; et, non typée mais à tenir fidèle à la réponse servie, la fonction `etat()` de
     `frontend/tests/e2e/backup-schedule.spec.ts:111-125`.
   - `prochaine_et_retard` (`admin_auto_backup_e2e.rs:1121`) assert `lastSuccessInFuture` **faux** dans les cas passés et
     **vrai** dans le cas futur (assertion `overdue` de la l. 1159) — **rouge avant** (`null`).
   - `BackupScheduleBanner.svelte` : `kind === 'overdue'` **et** `lastSuccessInFuture` → clé neuve
     `admin-backup-banner-future` (« La dernière sauvegarde automatique réussie est datée du { $date }, après l'heure du
     serveur : son horloge a reculé. », quatre locales) ; sinon le texte actuel. `bannerKind` inchangé. Vitest neuf.
   **Mutation M-b5** : `is_success_in_future` qui rend toujours `false` → le test e2e rouge.
5. **AC-b5 — L-B6 : la limite dite dans les deux unités.** `admin-backup-server-too-large` prend `{ $size }`
   (`formatBackupSize(importMaxMib × 1 048 576, locale)`) : « trop grande pour être restaurée par l'écran (limite
   { $mib } Mio, soit { $size }) » ; repli Svelte (`AutoBackupSection.svelte:454`) identique à la valeur fr-CH ;
   quatre locales (C-32-1-12). Le test « mention « trop grande » » (`AutoBackupSection.test.ts:239`) assert la présence
   de l'équivalent décimal (7 Mio → « 7,3 Mo » en fr-CH, valeur à confirmer au run et écrite telle qu'observée). La
   variable neuve est tenue par la garde existante `frontend/src/lib/shared/i18n-entrees-a-variables.test.ts`, qui
   couvre toute clé à variables : maillon 3 (« chaque site fournit les arguments que son repli réclame »), maillon 1
   (repli ↔ fr-CH), maillon 2 (« les quatre locales déclarent le même jeu de variables pour chaque clé ») — aucune garde
   neuve. **Mutation M-b7** : retirer `size` de l'appel `i18nMsg` du site → le maillon 3 rouge ; retirer `{ $size }` du
   seul `de-CH` → le maillon 2 rouge (journal de la seconde forme écrit aussi).
6. **AC-b6 — L-B7 : le refus du réglage traduit.** `update_backup_schedule` : `AppError::Validation(t(
   "admin-backup-schedule-error-out-of-domain", "<valeur fr-CH exacte>"))` ; clé dans les quatre catalogues ; entrée
   ajoutée à la table de G9 (`textes_coherents.rs`, `les_replis_rust_suivent_le_catalogue`, `TABLE: [(&str, Cles); 4]` →
   `5`, fichier `crates/kesh-api/src/routes/backup_schedule.rs`, un site, sans paramètre). Test neuf dans
   `crates/kesh-api/tests/i18n_e2e.rs` (C-32-1-13) : application montée en `DeCh` (`spawn_app_with_locale`), connexion
   administrateur, `GET` du réglage pour lire `version`, `PUT /api/v1/admin/backup-schedule` avec `intervalHours: 7`
   → **400**, `VALIDATION_ERROR`, message **égal** au rendu `de-CH` de la clé et **différent** du rendu `fr-CH` (marques
   d'isolation retirées, comme `audit_log_e2e.rs:396-405`). **Rouge constaté avant le correctif** (message français).
   **Mutation M-b6** : rétablir la chaîne française → rouge.
7. **AC-b7 — L-B3 et les autres phrases du manuel.** `docs/manual/fr/admin-manual.tex`, § *Sauvegarde automatique* :
   - *Espace disque requis* (l. 2349) : le seuil **ne mesure pas la base** ; il la compte à travers la dernière
     sauvegarde réussie ; à la **première**, une base plus grosse que le dossier des documents n'est couverte que par la
     marge de 256 Mio ; si l'espace manque alors, l'écriture échoue en « disque plein », le fichier partiel est supprimé,
     rien d'autre n'est touché. La **dernière phrase** du paragraphe (« Sur un NAS, la base partage souvent le volume du
     dossier de sauvegarde : ce seuil protège aussi la base. ») est réécrite pour ne plus contredire la précédente : le
     seuil ne mesure pas la base, mais en refusant d'écrire il évite de remplir le volume qu'elle partage ;
   - *Ce que l'écran montre* (l. 2345) : une modification du réglage non enregistrée est **conservée** par « Sauvegarder
     maintenant » ; si la réponse ne parvient pas (délai, coupure, 502, 504) ou si le serveur rend une erreur interne,
     l'écran relit l'état et la liste, où se lit le résultat (AC-b2) ;
   - *Le bandeau de l'application* (l. 2347) : un échec de lecture **garde** le dernier état lu ; et la phrase « « Aucune
     sauvegarde automatique réussie depuis le … » quand la dernière réussite date d'au moins deux intervalles » reçoit
     son cas futur (renvoi à *Horloge du serveur déréglée* et au texte de l'AC-b4) ;
   - *Horloge du serveur déréglée* (l. 2357) : le bandeau dit que la dernière réussite est datée **après** l'heure du
     serveur (texte de l'AC-b4), au lieu de « aucune sauvegarde réussie depuis le … » ;
   - *Les sauvegardes du serveur* (l. 2355) : la mention citée dans sa forme neuve (AC-b5).
   `make fr` sous verrou ; **contrôle aplati** du PDF (Dev Notes, § *Contrôle aplati d'un PDF* : apostrophe
   typographique normalisée, témoin positif d'abord) pour chaque phrase neuve, et `grep -c '??'` sur le texte aplati : 0.
   Les gardes qui lisent le manuel restent vertes (`binary(textes_coherents)`, `binary(configuration_transmise)`).
8. **AC-b8 — i18n.** Trois clés neuves (`admin-backup-auto-run-unknown`, `admin-backup-banner-future`,
   `admin-backup-schedule-error-out-of-domain`) et une modifiée (`admin-backup-server-too-large`, variable `$size`) dans
   les quatre catalogues ; `parity_between_locales` (`kesh-i18n/src/loader.rs`) vert ; G9 vert ;
   `npm run lint-i18n-ownership` vert (préfixe `admin-backup-` dans `features/admin-backup/`) ; les gardes frontend
   `frontend/src/lib/shared/i18n-entrees-a-variables.test.ts` (variable `$size`, AC-b5) et
   `i18n-repli-divergent-actif.test.ts` (repli Svelte de la clé modifiée égal au fr-CH) vertes. Recompte :
   `grep -c '^admin-backup-' crates/kesh-i18n/locales/*/messages.ftl` aux deux bornes, **+3** par locale.
9. **AC-b9 — recette visuelle** (thème clair, base de démonstration, `kesh-gate-logs/321-captures/`) : (a) formulaire
   modifié conservé après « Sauvegarder maintenant » ; (b) message de réponse manquante (délai simulé par une route
   interceptée en E2E, ou Vitest à défaut — dire lequel) ; (c) bandeau « datée après l'heure du serveur » ; (d) ligne
   « trop grande … soit … » ; (e) refus du réglage en allemand, si l'écran permet de le provoquer (sinon dire pourquoi).
   Jointes au Dev Agent Record et au prompt de revue.
10. **AC-b10 — E2E.** `frontend/tests/e2e/backup-schedule.spec.ts`, test « Administrateur : réglage gardé… » (l. 139) :
    avant « Sauvegarder maintenant », modifier le nombre conservé **sans enregistrer** ; après le 200, le champ garde la
    valeur saisie (AC-b1 à travers la frontière HTTP). Gate **complet**, les quatre gates frontend, **E2E complet au
    dernier commit de code**, jugé fichier par fichier contre `docs/testing.md` § *Les échecs attendus*.
11. **AC-b11 — mutations** M-b1 à M-b8 (huit ; M-b7 en deux formes ; même protocole que l'AC-a13).

### Tâches

- [ ] **T-b0** — Base et port propres (comme T-a0) ; dossiers E2E `/tmp/kesh-e2e-321/…` passés au backend **et** au runner.
- [ ] **T-b1 (AC-b6, AC-b8)** — clé × 4, test `i18n_e2e.rs` **rouge**, `t(…)`, G9.
- [ ] **T-b2 (AC-b4)** — tests rouges (unitaires, e2e), fonction, champ, bandeau, clé × 4.
- [ ] **T-b3 (AC-b1, AC-b2)** — Vitest rouges, `applyState`, prédicat d'issue inconnue extrait (AdminRestorePanel inchangé en
  comportement), `runNow` en trois cas, clé × 4 ; M-b1, M-b2, M-b8.
- [ ] **T-b4 (AC-b3)** — Vitest rouges, séquence, doc-comments.
- [ ] **T-b5 (AC-b5)** — clé modifiée × 4, repli Svelte, test ; M-b7 (deux formes).
- [ ] **T-b6 (AC-b7)** — manuel, `make fr` sous verrou, contrôle aplati ; après fusion de la 32-1c, mise à jour sur
  `main` et **régénération** du PDF (C-32-1-17).
- [ ] **T-b7 (AC-b9, AC-b10, AC-b11)** — E2E, captures, mutations, gates ; Dev Agent Record chiffré.

---

## C — 32-1c : la procédure de répétition (#714, hors ses deux choix tranchés)

**Modules** : manuels FR (`docs/manual/fr/admin-manual.tex`, `docs/manual/fr/user-manual.tex` pour ⚠️ seulement,
`docs/manual/shared/kesh-style.sty` pour la macro, et les **trois** PDF — la brochure se régénère aussi, sa règle du
`Makefile` dépendant du `.sty`, C-32-1-26) ; fiche `31-4-repetition-a-blanc.md` (annotations
datées) — **1** module (documentation). Aucun code. **La procédure ne se rejoue pas** : la 32-1c corrige ce que la
procédure dit, sans changer aucune commande ni ajouter aucune consigne d'agir (C-32-1-15).

### Critères d'acceptation

1. **AC-c1 — ce que la répétition écrit sur la production** (P4 LOW-1, LOW-3 ; `admin-manual.tex:1625`). La phrase ne
   dit plus « elle ne fait que lire la version » : de la production, la répétition **lit** la version et **télécharge**
   deux exports, qui y **écrivent** — deux lignes au journal d'audit, les sessions des connexions, et, si les réglages de
   facturation ou de relance n'ont jamais été enregistrés, leurs lignes aux valeurs par défaut ; pour la facturation,
   comme la première ouverture de sa page ; pour les **relances**, la première ouverture de la page pose **en plus** les
   trois niveaux par défaut, que l'export ne pose pas.
2. **AC-c2 — la cause d'un `realpath -e` en échec** (P3 E L-3 ; puce *Au contrôle~3*, l. 1905). La puce nomme d'abord la
   **barre finale** dans `COPIE` (contraire aux *Conventions*), puis le `/var/tmp` qui serait un lien. Le geste (« laissez
   le dossier vide ») **ne change pas** ; son remplacement est dans [#726] (point 1).
3. **AC-c3 — l'inventaire du contrôle 4 qui n'a rien lu** (P2 B2-L4 ; « Attendu » de la l. 1710). L'attendu dit que les
   deux `curl` ne disent rien, et qu'un message de `curl` ou de `grep` (fichier introuvable) est un écart — **un arrêt**,
   l'inventaire n'ayant alors rien lu (la règle « tout autre écart y est un arrêt » existe déjà : elle est appliquée, non
   créée).
4. **AC-c4 — les écritures à éviter entre les deux exports** (P2 E-LOW-7 ; étape 0, l. 1686). La liste nomme aussi une
   **intégration par clé d'API** qui écrit. « Refaire les exports en cas de doute » n'est pas écrit ([#726], point 2).
5. **AC-c5 — la limite du contrôle final** (P4 LOW-2 ; l. 1900). L'attendu dit que le contrôle final ne regarde que
   `$COPIE` : un dossier d'une autre répétition, resté sans objet Docker, n'y paraît pas ; c'est le
   `ls -d /var/tmp/kesh-repetition-*` (déjà écrit l. 1880) qui le montre.
6. **AC-c6 — « Ou, avec sudo : » tenu avec son listing** (P3 B3-2 ; l. 1851-1852). Le `\needspace{10\baselineskip}`
   passe **avant** l'amorce. Contrôle au PDF (`pdftotext -layout`) : l'amorce et la première ligne de son listing sur la
   même page ; aucun listing de la sous-section coupé. Capture de la page (rendu de la page du PDF en image,
   `kesh-gate-logs/321-captures/`), jointe au Dev Agent Record (recette visuelle : un contrôle visible).
7. **AC-c7 — ⚠️ imprimé** (P2 B2-L5, C-32-1-16). Macro `\keshAttention` dans `docs/manual/shared/kesh-style.sty`,
   définie sur `\faExclamationTriangle` (l. 37 — un `\providecommand` dont le rendu est un **« ! » gras**, fontawesome
   n'étant pas chargé) ; une macro sans argument avale l'espace qui la suit : chaque emploi s'écrit `\keshAttention\ `
   suivi du texte (comme le titre de `keshwarning`, `\faExclamationTriangle\ Attention`, l. 299), ou `\keshAttention{}`.
   Les **26 occurrences** (17 dans `admin-manual.tex`, sur 16 lignes — la l. 2923 en porte deux — et 9 dans
   `user-manual.tex`) sont remplacées **avec leur sélecteur U+FE0F** (la séquence `⚠️` entière, non le seul `⚠` : un
   U+FE0F orphelin resterait invisible à `grep "⚠"`). Contrôles : `grep -c "⚠" docs/manual/fr/*.tex` et
   `grep -c $'\xef\xb8\x8f' docs/manual/fr/*.tex` rendent 0 partout ; les PDF régénérés (`make fr`) portent le signe
   à ces 26 endroits — contrôle par échantillon de trois phrases aplaties **qui commencent par le signe** (par exemple
   « ! Un dossier = une instance », `admin-manual.tex:99`), nommées au Dev Agent Record, la phrase sans le signe ne
   distinguant rien — ; `grep -c '??'` sur les textes aplatis : 0. Capture d'une page portant le signe, jointe au Dev
   Agent Record.
8. **AC-c8 — la fiche de la 31-4 annotée, non réécrite** (P2 A-LOW-4, P4 LOW-4). Deux annotations datées
   (« *Annotation du 2026-10-2x, Story 32-1c* ») : à l'AC11 (`31-4-repetition-a-blanc.md:537-543`), G15 ne lit rien de la
   sous-section (elle écrit ses volumes en `\keshpath{…}`, G15 lit `\texttt{…}` et `/var/lib/docker/volumes/`) ; à la
   l. 504, le `test ! -e "$COPIE"` décrit n'est plus le contrôle final (renvoi à l'annotation de la l. 435). Le texte
   d'origine reste.
9. **AC-c9 — le manuel et ses gardes.** `make fr` sous verrou ; chaque phrase neuve des AC-c1 à AC-c5 contrôlée au PDF
   aplati (Dev Notes, § *Contrôle aplati d'un PDF*) ; la brochure régénérée garde son texte (`pdftotext` de
   `marketing-brochure.pdf` identique à celui de la version d'avant, extraite dans `kesh-gate-logs/` par
   `git show HEAD:docs/manual/fr/marketing-brochure.pdf` puis passée à `pdftotext` — C-32-1-26) ; gate ciblé `binary(textes_coherents)` et
   `binary(configuration_transmise)` (gardes G14 à G16, G19, qui lisent la sous-section) — C-32-1-18. Aucune commande de
   la sous-section ne change : `git diff` des `lstlisting` de `sec:repetition-cloture` **vide** (contrôle écrit au Dev
   Agent Record).
10. **AC-c10 — #714 sans constat propre.** Après la 32-1c, chaque ligne de #714 est soit corrigée (AC-c1 à AC-c8), soit
    dans [#726] (*Issues ouvertes*, n° 2), soit écartée avec sa raison (la moitié fausse de P3 E L-3).

### Tâches

- [ ] **T-c1 (AC-c1 à AC-c5)** — phrases ; **aucune** commande touchée.
- [ ] **T-c2 (AC-c6, AC-c7)** — `\needspace`, macro, 26 remplacements (`⚠️` entier), captures.
- [ ] **T-c3 (AC-c8)** — deux annotations datées.
- [ ] **T-c4 (AC-c9)** — `make fr`, contrôles aplatis, gate ciblé ; Dev Agent Record.

---

## Dev Notes

### Pourquoi trois clés, et dans quel ordre

Les trois parties se développent **séparément**, mais elles ne sont pas sans fichier commun (C-32-1-27) :
- **32-1a et 32-1b** partagent trois fichiers de code, en hunks distincts : `crates/kesh-api/src/auto_backup.rs` (A : les
  helpers lexicaux du `mod tests`, l. 1117-1201 ; B : `is_success_in_future` près de `is_overdue`, l. 221, et ses tests
  près de `is_overdue_dans_le_futur`, l. 950, dans le même `mod tests`), `crates/kesh-api/tests/admin_auto_backup_e2e.rs`
  (A : `TestEnv` l. 55-129 et `dossier_auto_inutilisable` l. 684 ; B : `prochaine_et_retard` l. 1121) et
  `crates/kesh-api/tests/textes_coherents.rs` (A : `DATE_ISO` l. 277 et un test neuf ; B : `TABLE` de G9, l. 694).
  Fusion textuelle sans conflit attendue ; la clé fusionnée **en second** se met à jour sur `main` (rebase), vérifie
  `git diff` des trois fichiers, et repasse son gate **complet** avant sa fusion.
- **32-1b et 32-1c** partagent `admin-manual.tex` (sections distinctes) et son PDF (binaire : conflit à toute fusion
  parallèle) : **32-1c avant 32-1b**, la 32-1b régénérant le PDF sur le `.tex` fusionné (C-32-1-17).
- Ordre : 32-1a quand elle est prête ; 32-1c avant 32-1b.

Mots-clés de PR (à poser par l'orchestrateur, **jamais dans un commit**) : 32-1a `closes #716` ; **`closes #711` porté
par celle des 32-1a et 32-1b qui fusionne EN DERNIER** (E-9 est dans la 32-1a, le reste dans la 32-1b ; E-1 et E-2 sont
dans [#725], déjà ouverte), l'autre portant `refs #711` — à décider au moment d'ouvrir la seconde PR, et à vérifier
après fusion (`gh issue view 711 --json state`) ; 32-1c `closes #714` ([#726] étant ouverte).

### Contrôle aplati d'un PDF (AC-b7, AC-c7, AC-c9)

`pdftotext` écrit l'apostrophe **typographique** `’` (U+2019) là où le `.tex` et la fiche ont l'apostrophe droite ; un
`grep -oF "… l'écran"` rend alors **rien** sur un PDF juste. La commande normalise par `sed` (GNU `tr` travaille par
octets et ne sait pas remplacer `’`, codé sur trois) :

```sh
pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | sed "s/’/'/g" | tr -s ' ' | grep -oF "<phrase>"
```

**Témoin positif d'abord**, sur le PDF d'avant le correctif : `"trop grande pour être restaurée par l'écran"` doit
sortir (vérifié le 2026-10-24 à la validation P1 ; sans le `sed`, rien). Une phrase neuve qui ne sort pas se recherche
par un fragment plus court avant de conclure (une coupure de mot en fin de ligne y laisse un tiret). Aucune espace
insécable n'est sortie du PDF actuel (`grep -c $'\xc2\xa0'` sur le texte aplati : 0) ; si une phrase neuve en introduit
une, l'ajouter au `sed`.

### 32-1a — remarques de conception

- **AC-a3** : le test lit les catalogues réels (`CARGO_MANIFEST_DIR/../kesh-i18n/locales`), comme le test des frais de
  rappel. La mutation M-a4 touche quatre `.ftl` : restauration par `git checkout -- crates/kesh-i18n/locales && touch`
  des quatre fichiers.
- **AC-a4** : chaque compte posé à la main doit être une valeur **légitime** du réglage — imputable, du type que
  `VAT_SETTING_ACCOUNT_TYPES` exige (`Liability`, `Asset`, `Liability`) — pour qu'aucun test ne prouve un cas que la
  route refuserait ; 2000, 1170 et 2201 le sont au plan PME (`pme.json` : comptes feuilles du bon type), ce que
  l'assertion de montage relit en base plutôt que de le supposer. Les `UPDATE` directs se font **après** le `commit`
  de (t1) : faits sur le pool pendant (t1), ils attendraient le verrou de la ligne de réglages tenu par la transaction.
- **AC-a9** : la sonde ne change pas `BackupEnv` (trait de production) ; elle vit dans le fichier de test.
- **AC-a11** : les helpers sont dans `mod tests` d'un fichier de production : leur modification ne change aucune ligne
  servie, mais le fichier est relu en revue comme du code. Le balayeur ne vise pas tout Rust (pas de `syn`,
  C-32-1-6) : il couvre les formes présentes dans les cinq fichiers lus (`main.rs`, `lib.rs`, `routes/admin.rs`,
  `routes/backup_schedule.rs`, `auto_backup.rs` — relevé à la validation P1 : chaînes brutes `r"…"` aux
  `auto_backup.rs:66`, `:72`, durées de vie `&'static` aux `auto_backup.rs:258` et `routes/backup_schedule.rs:194`,
  aucun littéral de caractère hors `mod tests`) ; son doc-comment le dit.

### 32-1b — remarques de conception

- **AC-b1** : « égalent le réglage lu » se compare sur les trois champs, `Number(intervalHours)` et
  `Number(retentionCount)` (les champs liés aux `<select>`/`<input>` sont des chaînes en Svelte). Un réglage modifié
  ailleurs (autre administrateur) pendant la saisie n'est pas un cas de l'AC : la `version` change, l'enregistrement rend
  409, et « Recharger » remplace la saisie — comportement existant, inchangé.
- **AC-b2** : trois cas (C-32-1-22) — issue inconnue (prédicat d'`issueInconnue`, extrait et partagé), autre 5xx,
  4xx (409 `BACKUP_IN_PROGRESS` compris) ; seuls les deux premiers relisent. La relecture n'appelle pas `loadSchedule`
  (qui remet `conflict`/`saveError` et réécrit le formulaire) mais `fetchBackupSchedule` + `applyState`, et pose
  `loadError` si elle échoue.
- **AC-b3** : le motif de séquence est celui de `fiscal-years` (C-29-1-37) ; la séquence est **module-level**, comme
  `_value`.
- **AC-b4** : `passe()` est privée ; `is_success_in_future` vit dans le même module et la réemploie (une seule règle de
  « futur »). Le bandeau ne compare toujours aucune date (doctrine du module).
- **AC-b6** : `i18n_e2e.rs` a son propre verrou (`get_i18n_lock`, l. 27) ; le test neuf le prend, comme ses voisins.
  Les routes d'administration exigent un administrateur **non PAT** (`require_not_pat`) : la connexion par mot de passe
  d'`ensure_admin_user` y suffit.

### 32-1c — remarques

- Le manuel fait foi pour la procédure **telle qu'elle a été jouée** (encadré « Ce que cette procédure n'a pas
  éprouvé », l. 1918-1932) : aucune phrase neuve ne doit affirmer un comportement **non joué** ; les AC-c1 à AC-c5 disent
  des faits du code (AC-c1), des faits du shell (AC-c2, AC-c3 : `realpath` retire la barre finale ; `grep` sur un fichier
  absent écrit sur la sortie d'erreur et rend 2) ou des limites du texte (AC-c4, AC-c5).
- L'encadré n'est **pas** mis à jour : aucun geste ne change.

### Issues ouvertes (textes de la fiche, ouvertes par l'orchestrateur le 2026-10-24)

*Les deux textes ci-dessous ont été rédigés avant l'ouverture et sont gardés tels quels pour la trace ; les issues
font foi.*

**1. [#725] — Sauvegarde automatique : un tour figé tient le verrou sans limite, et un `record_success` en erreur après
l'écriture n'a pas de chemin (E-1, E-2 de #711)** — labels `bug`, `P3`.

> Deux chemins d'erreur du tour de sauvegarde automatique (Story 31-1c), relevés en revue (E-1, E-2 de #711) et sortis
> de la Story 32-1b parce qu'ils demandent une conception, non un correctif mécanique.
>
> **1. Un tour figé tient `backup_lock` sans limite.** `run_once` prend le verrou en `try_lock_owned`
> (`crates/kesh-api/src/auto_backup.rs:669`) et le tient jusqu'à la fin, rétention comprise. Une écriture qui ne rend
> jamais la main (partage réseau gelé) bloque l'export et l'import, qui l'attendent en `lock_owned().await`
> (`crates/kesh-api/src/routes/admin.rs:77`, `:213`). Et tant qu'aucune réussite n'a jamais été enregistrée, rien ne le
> signale : `is_overdue` est faux sans `last_success_at` (`auto_backup.rs:221-226`), et le statut reste celui de la
> tentative précédente. À décider : un délai du tour (et ce qu'il fait d'une écriture en cours dans un
> `spawn_blocking`, qui ne s'annule pas), un délai d'attente de l'export et de l'import (409 plutôt qu'attente sans
> fin), un signal « tentative en cours depuis N » au panneau ou au bandeau.
>
> **2. `record_success` en erreur après l'écriture.** `record_success` (`auto_backup.rs:713`) en `Err` remonte par `?` :
> l'archive est écrite dans `auto/`, `last_success_*` ne bouge pas, la rétention ne tourne pas, `POST
> /api/v1/admin/backup-schedule/run` rend 500, et la reprise, une heure plus tard (`DELAI_DE_REPRISE`,
> `last_attempt_at` déjà posé), écrit une seconde archive. Chemin non testé. À décider : enregistrer un statut d'échec
> distinct, appliquer quand même la rétention en protégeant le fichier écrit, ou supprimer l'archive non enregistrée.
>
> Sans effet sur les comptes ni sur les données sauvegardées. Relevé le 2026-10-24 (Story 32-1, C-32-1-14). Priorité P3.

**2. [#726] — Manuel, répétition d'une clôture : gestes à trancher par le Project Lead (reste de #714)** — labels
`documentation`, `P3` (ouverte sans `bug` : aucun point ne décrit un comportement faux du produit).

> Constats de la revue de la Story 31-4 (§ *Répéter une clôture sur une copie*, `sec:repetition-cloture` de
> `docs/manual/fr/admin-manual.tex`) dont la correction **change une commande ou ajoute une consigne** — donc une
> procédure qui n'a pas été rejouée. Sortis de #714 par la Story 32-1c, qui a corrigé les phrases sans toucher aux
> gestes (C-32-1-15). Chacun attend un choix du Project Lead ; tant qu'il n'est pas fait, le manuel reste tel qu'il est.
>
> 1. **`realpath -e` en échec au contrôle 3** (`:1905`) : la cause la plus probable est une barre finale dans `COPIE`
>    (désormais nommée). Faut-il un geste — reposer `COPIE` sans barre (`COPIE=${COPIE%/}`), puis supprimer le dossier par
>    la garde — au lieu de « laissez le dossier vide » ? Le reste ne bloque pas une reprise (nouveau dossier suffixé) :
>    c'est un dossier vide que `/var/tmp` garde trente jours.
> 2. **Écriture intercalée entre les deux exports** (étape 0, `:1686`) : faut-il écrire « en cas de doute, refaites les
>    deux exports » ?
> 3. **Navigateur** : faut-il demander de refuser l'enregistrement des identifiants de la production pour l'adresse de la
>    copie (`127.0.0.1:18314`), et d'effacer de l'historique des téléchargements les entrées de la répétition ?
> 4. **Port pris entre le contrôle 8 et `up -d`** (`:1772`) : quelle branche d'arrêt — destruction complète (des
>    conteneurs ont pu être créés) ou troisième listing ?
> 5. **`mariadb:10.11` en dur dans la destruction** (`:1844`, `:1906`) : juste au tag 0.15.0 ; faut-il une consigne
>    « remplacez par l'image du compose de votre version », ou la lire dans le compose téléchargé ?
> 6. **Secrets en arguments de `sed`** (contrôle 5, `:1714-1720`) : visibles un instant par `ps` des autres utilisateurs
>    de la machine ; faut-il une autre écriture du `.env` (variables du shell, fichier temporaire `0600`) ?
> 7. **`export KESH_…_HOST_DIR=`** (contrôle 6, `:1726`) : la regex ne reconnaît pas une ligne préfixée par `export` ; le
>    cas ne se produit pas avec le `.env.example` du tag (aucun `export`), mais un `.env` venu d'ailleurs l'aurait.
>    Élargir la regex ?
>
> Relevé le 2026-10-24 (Story 32-1c). Priorité P3.

### Inventaire des sites non résolus

- **Classe « refus de validation en français seul »** (`AppError::Validation("…".into())` dans `crates/kesh-api/src/routes/`) :
  L-B7 n'en corrige **qu'un** (`backup_schedule.rs`) ; #711 note le « motif répandu dans `routes/` ». Les autres sites
  sont **hors périmètre** et **suivis par [#710]** (ouverte, P4 : « Les refus de validation du backend
  (AppError::Validation) sont écrits en français en dur dans les routes, hors journal d'audit » ; #469, fermée, ne
  couvrait que le journal d'audit). Le développement relève le compte aux deux bornes et l'écrit au Dev Agent Record,
  par un décompte **multiligne** (une forme sur deux lignes — `AppError::Validation(` en fin de ligne, chaîne à la
  suivante — échappe à un `grep` ligne à ligne) et en deux périmètres :
  `grep -rzoP 'AppError::Validation\(\s*"' crates/kesh-api/src/routes | tr '\0' '\n' | grep -c Validation` (chaîne
  littérale ; **145** au 2026-10-24, validation P1) et
  `grep -rzoP 'AppError::Validation\(\s*format!\(\s*"' crates/kesh-api/src/routes | tr '\0' '\n' | grep -c Validation`
  (chaîne formatée ; **101**). Attendu après la 32-1b : le premier baisse de **1**, le second ne bouge pas ; le décompte
  est versé en commentaire sur #710.
- **Mention de taille en Mio ailleurs** : `admin-backup-exceeds-import-limit` (`AdminBackupPanel.svelte:96-99`) dit
  « (N Mio) » sans afficher la taille du fichier à côté : pas de juxtaposition d'unités, hors classe de L-B6.
- **Date ISO** : `2026-02-30` reste une date pour `nombres_isoles` (AC-a7, limite écrite).
- **Gardes lexicales** : `corps()` reste coupée par un `\n}` dans une chaîne brute (AC-a11, écrit) ; aucune n'existe dans
  les fichiers lus (`routes/admin.rs`, `routes/backup_schedule.rs`, `auto_backup.rs`, `main.rs`, `lib.rs`) — à
  recontrôler au développement par `grep -n '^}' ` dans des chaînes, déclaré.
- **⚠️ hors des manuels FR** : `crates/kesh-report/src/pdf.rs:1532` écrit `⚠` dans un titre de PDF de rapport ; police et
  rendu non vérifiés ici — **hors** (code, autre police) ; à signaler à l'orchestrateur, issue si le glyphe manque au
  rendu.
- **E-9 (2)** : angle mort assumé (AC-a10).
- **Copies de l'assistant lexical `code()`** (même `take_while(… != "#[cfg(test)]")` ligne à ligne, sans connaissance
  des blocs `/* … */` ni des chaînes) : `crates/kesh-api/src/admin_backup/export.rs:442` et `:508`,
  `crates/kesh-api/src/routes/admin.rs:1097` (`code_de_production`), `crates/kesh-api/src/document_storage.rs:608`,
  `crates/kesh-api/src/routes/journal_entries.rs:1018` ; `crates/kesh-api/tests/audit_label_registry.rs` a sa propre lecture
  (`strip_line_comments`, l. 179 ; `source_assainie`, l. 231 ; `accolades_hors_chaines`, l. 291).
  L'AC-a11 ne corrige que celle d'`auto_backup.rs` (constat E-9 (3)) ; les cinq autres
  gardent la limite — **hors périmètre, non corrigées ici**, à tracer : issue demandée à l'orchestrateur (rapport de
  la remédiation P1), qui décidera aussi de l'extraction d'un balayeur commun (règle DRY).

### Fichiers touchés (prévus)

- **32-1a** : `crates/kesh-api/src/routes/invoice_pdf_service.rs` (commentaire, `mod tests`) ;
  `crates/kesh-api/src/auto_backup.rs` (`mod tests` seul) ; `crates/kesh-api/tests/audit_log_e2e.rs` ;
  `crates/kesh-api/tests/textes_coherents.rs` ; `crates/kesh-api/tests/admin_auto_backup_e2e.rs` ;
  `crates/kesh-db/src/repositories/company_invoice_settings.rs` (doc-comment) ;
  `crates/kesh-db/tests/company_invoice_settings_repository.rs` ;
  `frontend/src/routes/(app)/settings/fiscal-years/fiscal-years-page.test.ts`.
- **32-1b** : `frontend/src/lib/features/admin-backup/{AutoBackupSection.svelte, AutoBackupSection.test.ts,
  BackupScheduleBanner.svelte, BackupScheduleBanner.test.ts, backup-schedule.svelte.ts, backup-schedule.api.ts,
  backup-schedule.api.test.ts}` ; `frontend/src/lib/features/admin-restore/AdminRestorePanel.svelte` et un module
  partagé neuf sous `frontend/src/lib/shared/utils/` avec son test (prédicat d'issue inconnue extrait, AC-b2) ;
  `frontend/tests/e2e/backup-schedule.spec.ts` ; `crates/kesh-api/src/auto_backup.rs` ;
  `crates/kesh-api/src/routes/backup_schedule.rs` ; `crates/kesh-api/tests/{i18n_e2e.rs, admin_auto_backup_e2e.rs,
  textes_coherents.rs}` ; `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl` ;
  `docs/manual/fr/admin-manual.{tex,pdf}`.
- **32-1c** : `docs/manual/fr/admin-manual.{tex,pdf}` ; `docs/manual/fr/user-manual.{tex,pdf}` ;
  `docs/manual/fr/marketing-brochure.pdf` (régénéré par la dépendance au `.sty`, texte inchangé, C-32-1-26) ;
  `docs/manual/shared/kesh-style.sty` ; `_bmad-output/implementation-artifacts/31-4-repetition-a-blanc.md`.
- **Toutes** : cette fiche, `sprint-status.yaml`, le registre `-321`.
- **CHANGELOG** (C-32-1-26) : la 0.15.0 est publiée (`## [0.15.0] — 2026-10-24`, `CHANGELOG.md:11`) et aucun bloc non
  publié n'existe au 2026-10-24. Convention du dépôt (celle que `scripts/prepare-release.sh` date, l. 237-255) : un
  bloc `## [0.16.0] — Non publié` en tête, sous la ligne `---`. La clé qui fusionne la première le **crée** s'il n'existe
  pas encore sur `main` (une autre story de l'Epic 32 a pu le faire), les suivantes y ajoutent ; rubrique **`### Corrigé`**
  (le CHANGELOG est en français : `Ajouté`, `Modifié`, `Corrigé`, `Interne, invisible de l'utilisateur`). Une ligne pour
  la 32-1b (écran de la sauvegarde automatique) et une pour la 32-1c (manuels : procédure de répétition, signe
  d'attention imprimé) ; la 32-1a n'a aucun effet visible — pas d'entrée.

### Risques

- **32-1a** : faible ; le risque est le **test qui passe à vide** (`memory/tests-qui-prouvent-moins`) — d'où un rouge
  constaté avant chaque correctif et une mutation tuée par AC.
- **32-1b** : moyen ; trois changements de comportement d'écran, dont un état partagé (bandeau) ; l'E2E et les captures
  sont exigés.
- **32-1c** : la documentation exécutée a coûté 8 + 4 passes à la 31-4 ; ici, **aucune commande ne change** (AC-c9, diff
  des listings vide), ce qui borne la revue à des phrases.

### Références

- `_bmad-output/planning-artifacts/epic-32-serveur-mcp.md` § *32-1* ; registre `epic-32-choix-autonomes.md` (C-32-0-3).
- Issues #711, #714 (et le commentaire de Guy du 2026-10-24), #716 ; relevés `kesh-gate-logs/311c-review-p1-E.md`,
  `315-low-residuels.md`, `314-review-p4.md`.
- Fiches `31-1-sauvegarde-complete.md` (31-1c, AC25-AC40, C-31-1-55, C-31-1-93), `31-4-repetition-a-blanc.md`,
  `31-5-defauts-courts.md` (tableau des mutations).
- `CLAUDE.md` § *Review Iteration Rule*, *Recette visuelle*, *Exception kesh-db*, *Propagation post-patch*.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-10-24 — Fiche rédigée (Opus 5.5) ; trois clés à `ready-for-dev` ; registre C-32-1-1 à C-32-1-19 ; prompt de
  validation P1 `321-validate-prompt-p1.md`.
- 2026-10-24 — **Validation P1** (Sonnet 5.5, deux lentilles en contexte frais, lecture seule ; rapports
  `kesh-gate-logs/321-validate-p1-R.md` et `321-validate-p1-F.md`). Lentille R : **0 CRITICAL, 0 HIGH, 3 MEDIUM,
  11 LOW** (R1 à R14) ; lentille F : **0 CRITICAL, 0 HIGH, 3 MEDIUM, 10 LOW** (F1 à F13). Recoupements : R1 = F1,
  R3 = F2, R6 = F3, R7 = F9, R8 = F6, R9 = F5, R10 = F7, R11 = F8 = F10 — soit **18 constats distincts**, dont **4
  MEDIUM** (R1, R2, R3, R6 = F3, classé à la plus haute des deux sévérités). Remédiation (Opus 5.5), chaque affirmation
  relue au code :
  - **R1 = F1** (MEDIUM) — AC-a11 : balayeur à états partagé par `code()` et `argument_de` (chaînes, brutes comprises,
    `//`, `/* */` imbriqués, littéraux `'x'` distingués des durées de vie) ; assertions de montage tirées de
    `lib.rs:1186` (`/*` dans une chaîne, aucun `*/`, aucun `#[cfg(test)]`) et du vrai `lib.rs` (`.with_state(state)`,
    l. 1200) ; mutation M-a7 (retrait naïf). C-32-1-20.
  - **R2** (MEDIUM) — constat E7 et AC-a8 : séquence établie au code — trois verrous de TVA, trois rôles, puis jusqu'à
    quatre comptes désignés par `chart_designated_accounts` / `designated_account_id` (l. 1118, 512) ; au plan PME
    1171, 2200, 2206, 1100, 3000, 2000, 6940, 3800, 6900, 3805 ; le doc-comment n'affirme pas de liste close. C-32-1-23.
  - **R3 = F2** (MEDIUM) — AC-a4 : les trois colonnes posées (2000, 1170, 2201, relues en base), deux tests, M-a1a/b/c ;
    `UPDATE` directs sur le pool **après** le `commit` de la transaction qui crée la ligne. C-32-1-21.
  - **R6 = F3** (MEDIUM par F, LOW par R) — AC-b2 : « pas parvenue » réservé à l'issue inconnue (prédicat
    d'`issueInconnue`, extrait et partagé : `TIMEOUT`, `NETWORK_ERROR`, 502, 504) ; autre 5xx : message du serveur et
    relecture ; 4xx : message du serveur sans relecture ; texte corrigé (« l'écran relit … : vérifiez-les ») ; M-b8.
    C-32-1-22.
  - **R4** — AC-a9 : `free_bytes` seul (le compteur de `wrap_writer` était inerte : `ENOTDIR` avant `remplir`).
    C-32-1-25. **R5** — `is_success_in_future` exige `enabled`, cas « désactivée » ajouté. C-32-1-24. **R7 = F9** —
    fixtures nommées (`backup-schedule.api.test.ts:44`, `AutoBackupSection.test.ts:45`,
    `BackupScheduleBanner.test.ts:43`, `etat()` de l'E2E). C-32-1-28. **R8 = F6** — Dev Notes § *Contrôle aplati d'un
    PDF* : `sed "s/’/'/g"` (GNU `tr` ne sait pas, par octets), témoin positif vérifié ; AC-b7, AC-c9 et prompt P2.
    **R9 = F5** — 17 + 9 = 26 occurrences, `⚠️` avec U+FE0F, contrôle du sélecteur. **R10 = F7** — fichiers communs
    A/B nommés, rebase et gate complet de la seconde, `closes #711` sur la dernière fusionnée. C-32-1-27. **R11 = F8 =
    F10** — brochure régénérée et commitée (texte contrôlé inchangé) ; CHANGELOG `## [0.16.0] — Non publié`, `### Corrigé`.
    C-32-1-26. **R12** — #710 cité, décompte multiligne (145 chaînes littérales, 101 formatées). **R13** — AC-b7 nomme
    les phrases des l. 2347 et 2349 ; AC-b8 nomme `i18n-entrees-a-variables.test.ts` et
    `i18n-repli-divergent-actif.test.ts`. **R14** — « dix constats, douze lignes ».
  - **F4** — inventaire des cinq autres copies de l'assistant lexical (et de la lecture d'`audit_label_registry.rs`) ;
    **non corrigées**, issue demandée à l'orchestrateur. **F11** — AC-c7 : emploi `\keshAttention\ `, signe « ! »
    gras, phrases témoins qui commencent par le signe, captures (AC-c6, AC-c7). **F12** — échec de la relecture de
    l'état : `loadError`. **F13** — **en partie réfuté** : la garde existante `i18n-entrees-a-variables.test.ts` tient
    déjà la variable (maillons 2 et 3) ; nommée, et mutation M-b7 (deux formes) ajoutée ; aucune garde neuve.
  - Issues : « à ouvrir » remplacé par **[#725]** (E-1, E-2) et **[#726]** (gestes de #714) ; #710 pour la classe des
    refus en français. Décompte : −3 + 2 issues, −2 + 1 bugs. Registre : C-32-1-20 à C-32-1-28, amendements datés de
    C-32-1-2, 4, 6, 7, 9, 11, 14, 15, 16, 17. Propagation grepée dans la fiche et le registre (`issue pour Guy`,
    `à ouvrir`, `\b25\b`, `trois lectures`, `M-a1\b`, `M-b6`). **Quatre MEDIUM remédiés : une passe P2 complète est
    due** (`321-validate-prompt-p2.md`, Opus).

[#711]: https://github.com/guycorbaz/kesh/issues/711
[#714]: https://github.com/guycorbaz/kesh/issues/714
[#716]: https://github.com/guycorbaz/kesh/issues/716
[#710]: https://github.com/guycorbaz/kesh/issues/710
[#725]: https://github.com/guycorbaz/kesh/issues/725
[#726]: https://github.com/guycorbaz/kesh/issues/726

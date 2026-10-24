# Story 32.1 : défauts courts — tests et commentaires (32-1a), écran de la sauvegarde automatique (32-1b), procédure de répétition (32-1c), serveur de la sauvegarde automatique (32-1d) (#716, #711, #714)

Status: ready-for-dev

*Fiche rédigée le 2026-10-24, en autonomie (Opus 5.5), worktree `kesh-32-1`, branche `story/32-1` sur
`chore/kickoff-epic-32` (`6f0cee00`). Une fiche, **quatre** clés (C-32-0-3, C-32-1-1 ; la quatrième, 32-1d, née du
découpage de la 32-1b avant la validation P3, C-32-1-37) : `32-1a-defauts-tests-commentaires`,
`32-1b-defauts-sauvegarde-automatique` (l'écran), `32-1c-defauts-procedure-repetition`,
`32-1d-defauts-sauvegarde-serveur` (le serveur). Registre : `epic-32-choix-autonomes-321.md` (C-32-1-1 à C-32-1-38 ;
C-32-1-20 à 28 de la validation P1, C-32-1-29 à 36 de la validation P2, C-32-1-37 du découpage, C-32-1-38 de la
validation P3). Clé courte : `321`.
Action A8 de la rétrospective de l'Epic 31.*

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
si tout se passe comme écrit : **#716** fermée par la 32-1a ; **#711** fermée par la dernière fusionnée des 32-1a, 32-1b
et 32-1d — en pratique la dernière des 32-1a et 32-1b, la 32-1d fusionnant avant la 32-1b (C-32-1-27, C-32-1-37) —, E-1 et
E-2 sortis dans **[#725]** (ouverte le 2026-10-24), **E-9 (3)** renvoyé à **[#727]** (validation P3, C-32-1-38 : la
PR qui ferme #711 l'écrit) ; **#714** fermée par la 32-1c, les
gestes à trancher sortis dans **[#726]** (ouverte le 2026-10-24) ; les copies de l'assistant lexical que la 32-1a ne
corrige pas sorties dans **[#727]** (ouverte le 2026-10-24 sur la demande de la remédiation P1, *Inventaire*). Bilan :
**−3 + 3 = 0** issue ; les bugs : #711 et #714 portent `bug`, #716 non ; #725 porte `bug` (`bug`, `P3`), #726 non
(`documentation`, `P3`), #727 non (`technical-debt`, `P3`) — **−2 + 1 = −1** bug (étiquettes lues par l'orchestrateur à
l'ouverture, recoupées par les validations P1 et P2). Ce n'est pas un motif pour taire un constat (*On trace toujours*,
`CLAUDE.md` § *Priorités des défauts*) : la remédiation P2 n'ouvre aucune issue neuve — ce qu'elle sort va en
commentaire de [#725] et de [#727] (C-32-1-30, C-32-1-35) ; la remédiation P3 non plus — E-9 (3) va en commentaire de
[#727], qui porte déjà la classe (C-32-1-38). Le bilan ne change pas.

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
| E-9 (3) | **Vrai.** `code()` (`auto_backup.rs:1121-1128`) n'écarte que les lignes `//` ; `argument_de` (`:1159-1179`) ne voit pas les chaînes ; `corps()` (`:1182-1188`) coupe au premier `\n}`. | **hors** — [#727] (retiré de la 32-1a à la validation P3, C-32-1-38 ; angle mort écrit, AC-a10) |
| L-B3 | **Vrai ; site corrigé.** `seuil_d_espace` (`auto_backup.rs:496-500` — non les l. 490-491 citées, qui sont la fin de `taille_des_documents`) : `max(dernière, documents) × 1,2 + 256 Mio` ; la base n'est mesurée qu'à travers la dernière sauvegarde. Le manuel (`admin-manual.tex:2349`, § *Espace disque requis*) ne le dit pas. | 32-1b |
| L-B5 | **Vrai.** `is_overdue` vrai pour une réussite future (`auto_backup.rs:221-226`, voulu) ; `since` (`BackupScheduleBanner.svelte:33`) affiche la date future. | 32-1d (le drapeau servi) ; 32-1b (le bandeau qui le lit) — C-32-1-37 |
| L-B6 | **Vrai.** `formatBackupSize` en unités décimales (`backup-format.ts:14-28`) ; `exceedsImportLimit` en Mio (`backup-schedule.api.ts:129-131`) ; mention `AutoBackupSection.svelte:450-458`. | 32-1b |
| L-B7 | **Vrai.** `AppError::Validation("Réglage de sauvegarde hors domaine : …".into())` (`routes/backup_schedule.rs:133-137`). | 32-1d |

### #714 — constats de la revue de la 31-4 (manuel d'administration, `sec:repetition-cloture`, l. 1617-1933)

Les **deux choix de procédure** sont tranchés par Guy et versés en v0.15.0 (commentaire du 2026-10-24) : vérifié,
`admin-manual.tex:1791` (l'écart de pièces est un arrêt) et `:1906` (le `pull` en échec, le troisième listing). Ils ne
reviennent pas. Règle de partage des autres (C-32-1-15) : **la 32-1c corrige ce que la procédure dit** — une cause, une
sortie attendue, une source d'écriture, une limite —, **sans** ajouter, retirer ni changer **aucune commande** ni
**aucune consigne d'agir** ; le reste va dans [#726], pour Guy.

| Id (#714) | Établi au code | Clé |
|---|---|---|
| P3 E L-3 — `realpath -e` en échec | **En partie vrai.** « En cas d'arrêt », puce *Au contrôle~3* (`:1905`) : « laissez le dossier vide, et traitez la cause (un `/var/tmp` qui serait un lien) ». La cause la plus probable est une **barre finale** dans `COPIE` (`realpath` la retire, la comparaison échoue), que la puce ne nomme pas ; *Conventions* (`:1635`) dit `COPIE` « toujours écrit sans barre finale ». « Un reste qui **bloque** une reprise le même jour » est **faux** : la reprise se fait dans un nouveau dossier, suffixé (`:1635`) ; le reste est un dossier vide. | cause : 32-1c ; geste (retirer la barre et supprimer par la garde) : **hors** — [#726] |
| P2 B2-L4 = E-LOW-2 — inventaire du contrôle 4 muet | **Vrai en partie.** Si un `curl -fsSL … -o` échoue (`:1704-1705`), le `grep` lit un fichier absent et la boucle n'affiche **rien sur la sortie standard** ; `curl` (`-S`) et `grep` écrivent pourtant leur erreur sur la sortie d'erreur — le terminal n'est donc pas muet, mais l'attendu (« rien ») ne dit pas que ce message compte comme écart, et une lecture qui ne regarde que la liste attendue le laisse passer ; le contrôle 7 rattrape. (« Muet » nuancé à la validation P2, F12.) | 32-1c (sortie attendue) |
| P2 E-LOW-7 — clients par clé d'API | **Vrai.** Étape 0 (`:1686`) : « ni saisie, ni import, ni autre utilisateur au travail » — une intégration par clé d'API qui écrit n'est pas nommée. « Refaire les exports en cas de doute » est une consigne neuve. | nommer : 32-1c ; refaire : **hors** — [#726] |
| P1 B-10 — gestionnaire de mots de passe, historique | **Vrai.** Rien n'avertit que le navigateur propose d'enregistrer, pour `127.0.0.1:18314`, les identifiants de la production saisis à la reconnexion (`:1788`), ni que l'historique des téléchargements garde les noms de fichiers. | **hors** — [#726] (consignes neuves) |
| P1 E LOW-2 — port pris entre le contrôle 8 et `up -d` | **Vrai.** Aucune branche ne dit ce que fait un `up -d` qui échoue (contrôle 9, `:1772`). | **hors** — [#726] (choix de branche) |
| P1 E LOW-3 — `mariadb:10.11` en dur dans la destruction | **Vrai.** `:1844` et `:1906` ; juste au tag 0.15.0 (`docker-compose.yml:4`), faux si un compose ultérieur change d'image. | **hors** — [#726] (consigne « remplacez ») |
| P1 E LOW-4 — secrets en arguments de `sed`, visibles de `ps` | **Vrai.** Contrôle 5, `:1714-1720`. | **hors** — [#726] (commande à changer) |
| P1 E LOW-5 — `export KESH_…_HOST_DIR=` non reconnu au contrôle 6 | **Vrai à la regex** (`:1726`), **sans cas réel** : le `.env` vient de `.env.example` du tag (`.env.example:235-266` : lignes commentées, aucun `export`), modifié par le seul `sed` du contrôle 5. | **hors** — [#726] (commande à changer) |
| P4 LOW-2 — branche « date inconnue » ; dossier sous un autre nom | **En partie vrai.** La branche « date inconnue » (`:1914`) est dans la même puce que le `ls -d` qu'elle suppose (premier geste de la puce *Reste d'une répétition antérieure*, `:1907`) : tenue. Le contrôle final (`:1880-1900`) ne regarde que `$COPIE` : un dossier d'une **autre** répétition, sans objet Docker, lui échappe — limite non écrite. | limite : 32-1c |
| P4 LOW-1, LOW-3 — « elle ne fait que lire la version » | **Vrai.** `:1625` : « elle ne fait que lire la version et télécharger deux exports » contredit la suite de la phrase, qui dit les écritures. Et « comme le fait la première ouverture de leur page » est inexact pour les **relances** : l'export appelle `company_dunning_settings::get_or_create_default` (`crates/kesh-api/src/exports/global.rs:313`, un `INSERT IGNORE` de la seule ligne), la page des relances appelle `ensure_seeded_in_tx` (`routes/company_dunning_settings.rs:56`), qui pose **en plus** les trois niveaux par défaut (`crates/kesh-db/src/repositories/company_dunning_settings.rs:168-200`). | 32-1c |
| P3 B3-2 — « Ou, avec sudo : » détaché | **Vrai.** `:1851-1852` : l'amorce précède le `\needspace{10\baselineskip}`, qui ne la retient donc pas avec son listing. | 32-1c |
| P2 B2-L5 — ⚠️ non imprimé | **Vrai, et plus large.** `grep -c "⚠"` (des **lignes**) : **16** dans `admin-manual.tex`, **9** dans `user-manual.tex`, 0 dans la brochure ; en **occurrences** (`grep -o "⚠" … \| wc -l`) : **17** et **9**, soit **26** (la l. 2923 d'`admin-manual.tex` en porte deux), toutes écrites `⚠️` (U+26A0 suivi du sélecteur U+FE0F : `grep -o $'\xe2\x9a\xa0\xef\xb8\x8f' docs/manual/fr/*.tex \| wc -l` rend 26), aucune dans un `lstlisting` ; `pdftotext … \| grep -c "⚠"` rend **0** pour les deux PDF (police TeX Gyre Heros, `kesh-style.sty:95`) — **avant comme après** tout correctif : ce contrôle ne prouve rien. **Recensé par le symptôme** (validation P2, F3 = R2) : un caractère sans glyphe dans la police sort de `pdftotext` en **U+FFFD** — `pdftotext docs/manual/fr/<pdf> - \| grep -o $'\xef\xbf\xbd' \| wc -l` rend **26** (manuel d'administration), **9** (manuel utilisateur), **16** (brochure). Ventilation, au `.tex` : administration = 17 `⚠️` + **9 signes de listings** dans TeX Gyre Cursor (`✓` l. 1547 ; `═══ … ═══` l. 3313, six ; `✓` l. 3318 ; `✗` l. 3320 — `grep -n '[✓✗═⇄]'`) ; utilisateur = ses 9 `⚠️` ; brochure = **15** `\faCheckCircle` (`\providecommand{\faCheckCircle}{\textbf{✓}}`, `kesh-style.sty:39`) + **1** `\faExchangeAlt` (`\textbf{⇄}`, l. 42 ; « Import bancaire automatisé »). Ni TeX Gyre Heros ni TeX Gyre Cursor n'ont U+2713, U+2717, U+21C4, U+2550, U+26A0 (table `cmap` des fichiers `.otf` de TeX Live, lue à la validation P2) ; tous deux ont `+`, `x`, `=`, `→` (U+2192, déjà employé par `\faRocket`). Même cause, même fichier (`kesh-style.sty`). | 32-1c, les **trois** PDF (C-32-1-16, C-32-1-31) |
| Fiche P2 A-LOW-4 — G15 et l'AC11 | **Vrai.** L'AC11 de la 31-4 (`31-4-repetition-a-blanc.md:537-543`) range G15 parmi « les gardes existantes qui relisent la sous-section » ; G15 (`configuration_transmise.rs:2732-2745`) lit `/var/lib/docker/volumes/<x>` et « volume `\texttt{<x>}` » — la sous-section écrit ses volumes en `\keshpath{…}` : G15 n'y lit rien. | 32-1c (annotation datée) |
| Fiche P4 LOW-4 — `test ! -e` au présent | **Vrai.** `31-4-repetition-a-blanc.md:504` (puce *nom faux*, AC7 (d)) décrit au présent un test que le manuel n'écrit plus ; la l. 433 est annotée (l. 435), la l. 504 non. | 32-1c (annotation datée) |

**Prémisses corrigées**, à signaler sur les issues (C-32-1-19) : #711 E-7 (site l. 95-100, non 111), L-B3 (l. 496-500,
non 490-491), E-9 (1) (distinction observable) ; #714 P3 E L-3 (« bloque une reprise » : faux) ; #716 E6 (l. 277-278 et
296) ; fichier d'epic, 32-1a : « M4 de la 31-5 tuée » — la M4 de la 31-5 est **équivalente** et le reste ; c'est la
mutation du `COALESCE` qui se tue (C-32-1-2).

---

## A — 32-1a : tests et commentaires (#716 entier ; E-9 (1) et (2) de #711, l'angle mort de E-9 (3) écrit)

*E-9 (3) — réécrire les helpers lexicaux d'`auto_backup.rs` — est **sorti de la 32-1a** à la validation P3 et renvoyé à
[#727] (C-32-1-38) : trois rédactions de l'AC-a11, trois défauts, chaque remédiation introduisant le suivant. Le numéro
AC-a11 est gardé avec une ligne de renvoi ; M-a7 n'est pas réemployé.*

**Modules** : `kesh-api` (tests : `audit_log_e2e.rs`, `textes_coherents.rs`, `admin_auto_backup_e2e.rs` ; `src` : un
commentaire de `routes/invoice_pdf_service.rs` et son `mod tests`, des doc-comments du `mod tests` d'`auto_backup.rs`),
`kesh-db` (doc-comment de `repositories/company_invoice_settings.rs`, `tests/company_invoice_settings_repository.rs`),
frontend (`settings/fiscal-years/fiscal-years-page.test.ts`) — **3** au grain des crates et paquets ; **6** au grain des
modules métier de premier niveau, compté comme pour la 32-1b (un fichier de test compte au module qu'il exerce) :
`kesh-api/routes/invoice_pdf_service`, `kesh-api/routes/audit_log` (par `audit_log_e2e.rs`), la garde des textes
(`textes_coherents.rs`), `kesh-api/auto_backup` (par `admin_auto_backup_e2e.rs` et les doc-comments de son `mod tests`),
`kesh-db/repositories/company_invoice_settings`, `frontend/settings/fiscal-years`. ⚠️ **Au-delà des cinq de la règle de
splitting préventif : signal déclaré au Project Lead** (C-32-1-38), sans découpage par la remédiation — les AC de la
32-1a sont indépendants deux à deux (chacun touche un module, aucun ne lit le résultat d'un autre), et le seul
recyclage de la story, l'AC-a11, en est sorti. 🗄️ (un test de dépôt `kesh-db`) : **gate complet même en
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
   - **`designate_vat_fills_nulls_and_is_idempotent`** — sans (u) : assertion de montage **après (t1)** — les trois
     colonnes sont `NULL` (sans quoi « remplit les `NULL` » passerait à vide si `insert_with_defaults_in_tx` venait à les
     poser) ; après (t2), les trois colonnes égalent les comptes
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
10. **AC-a10 — E-9 (2) et E-9 (3) : les angles morts des gardes lexicales, assumés et écrits.** Commentaires seuls,
    dans le `mod tests` d'`auto_backup.rs`, sans toucher une ligne de code des helpers :
    - doc-comment de `gardes_possedes` (`auto_backup.rs:1190-1201`) : la propriété tenue par le déplacement du garde de
      `full_export` dans `remplir` (le verrou vit jusqu'à la fin de l'assemblage même si le client part) ne s'observe
      qu'au départ du client pendant l'assemblage ; aucun test comportemental ne la tient (il faudrait un crochet de
      synchronisation dans `build_keshbackup`) — angle mort assumé, la garde lexicale (M-c24) est le seul filet
      (C-32-1-5) ;
    - doc-comment de `code()` (`auto_backup.rs:1119-1120`), d'`argument_de` (`:1157-1158`) et de `corps()` (`:1181`)
      (C-32-1-38) : chacun dit sa limite, telle que le constat E-9 (3) l'établit — `code()` n'écarte que les lignes
      entières qui commencent par `//` et lit comme du code un commentaire de fin de ligne, un commentaire `/* … */` et
      le contenu des chaînes ;
      `argument_de` compte les parenthèses écrites dans une chaîne ou un littéral de caractère ; `corps()` coupe au
      premier `\n}` du texte, chaîne comprise, et repose sur rustfmt (un item de tête se ferme par `}` en colonne 0) —,
      la conséquence muette (un jeton **exigé** écrit seulement dans un commentaire de bloc ou une chaîne satisfait la
      garde à tort ; un jeton interdit l'y fait rougir, ce qui est bruyant), et le renvoi à [#727] (extraction d'un
      balayeur commun). Aucun test, aucune mutation : le texte des helpers et leurs assertions de montage existantes
      ne changent pas, `lancement_par_main_seulement` et `gardes_possedes` restent verts sans y toucher.
11. **AC-a11 — retiré à la validation P3** (C-32-1-38). E-9 (3) — faire lire aux helpers lexicaux d'`auto_backup.rs`
    ce qu'ils disent — n'est plus corrigé par la 32-1a : il est **renvoyé à [#727]**, dont l'extraction d'un balayeur
    commun corrigera toutes les copies d'un coup (texte du commentaire à verser : *Commentaires à verser*). Motif :
    trois rédactions de cet AC — d'origine (retrait des `/* … */`, C-32-1-6), remédiation P1 (balayeur à états,
    C-32-1-20), remédiation P2 (réemploi de `lexique`, inclus « en tête » d'`auto_backup.rs`, C-32-1-32) —, trois
    défauts relevés aux validations P1 (un `/*` dans une chaîne de `lib.rs:1186`), P2 (M-a7 indécidable) et P3
    (l'inclusion en tête tronquait `code()` avant tout le code de production), chaque remédiation introduisant le
    suivant — le recyclage que la règle de splitting préventif désigne. Ni `crates/kesh-db/tests/lexique/mod.rs`
    ni le code des helpers (`code()`, `argument_de`, `corps()`) ne sont touchés par la 32-1a ; leur limite est **écrite**
    à leurs doc-comments (AC-a10), angle mort assumé d'ici l'extraction. Les mutations M-a7a et M-a7b sortent avec cet
    AC ; leur numéro n'est pas réemployé.
12. **AC-a12 — gates.** Gate **complet** (`scripts/test-fast.sh` sous verrou, base remise à zéro d'abord) ; les quatre
    gates frontend ; **E2E complet au dernier commit de code** (C-32-1-18), jugé fichier par fichier contre
    `docs/testing.md` § *Les échecs attendus*. Ne déclarer que ce qui a tourné.
13. **AC-a13 — mutations** (sous verrou, appliquées par Edit, restaurées par `git checkout -- f && touch f`, diff
    résiduel 0, un journal `321-mut-<id>.log` chacune dans `kesh-gate-logs/`) : M-a1a, M-a1b, M-a1c, M-a2 à M-a6
    (**huit** ; M-a7 est sortie avec l'AC-a11, C-32-1-38), chacune tuée par un test **nommé**. Une mutation qui survit
    est un défaut du test, non un constat à reporter.

### Tâches

- [ ] **T-a0** — Base `kesh_321`, `kesh_e2e_321`, port attribué par l'orchestrateur,
  `CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh-32-1/target` (`docs/agents-orchestration.md` § 4).
- [ ] **T-a1 (AC-a7)** — test de `nombres_isoles` écrit, **rouge constaté**, puis la regex ; `binary(textes_coherents)`.
- [ ] **T-a2 (AC-a3)** — test du préfixe ; M-a4 jouée (rouge), restaurée.
- [ ] **T-a3 (AC-a4)** — deux tests de dépôt ; M-a1a, M-a1b, M-a1c ; gate complet (kesh-db).
- [ ] **T-a4 (AC-a5)** — cas Vitest ; M-a2.
- [ ] **T-a5 (AC-a2, AC-a6)** — `audit_log_e2e.rs` ; M-a5, M-a6.
- [ ] **T-a6 (AC-a9)** — sonde ; M-a3. *(L'AC-a11 est retiré, C-32-1-38.)*
- [ ] **T-a7 (AC-a1, AC-a8, AC-a10)** — commentaires : B1, E7, `gardes_possedes`, et les limites de `code()`,
  `argument_de`, `corps()` avec le renvoi à [#727] (aucune ligne de code des helpers).
- [ ] **T-a8 (AC-a12, AC-a13)** — gates complets, E2E complet ; Dev Agent Record chiffré.

---

## B — 32-1b : l'écran de la sauvegarde automatique (#711 : E-6, E-7, E-8, L-B3 (manuel), L-B5 (bandeau), L-B6)

*Découpée avant la validation P3 (C-32-1-37) : le serveur — le drapeau `lastSuccessInFuture` de L-B5 et le refus traduit
de L-B7 — est passé dans la **32-1d** (partie D). Les numéros des AC et des mutations de la 32-1b sont **gardés** ; ce qui
est parti laisse une ligne de renvoi (AC-b6 ; M-b5, M-b6), pour que les références du registre et des rapports P1 et P2
restent lisibles.*

**Modules** : frontend (trois dossiers de `src/lib/` — `features/admin-backup/` ; `features/admin-restore/`, pour le seul
prédicat extrait (AC-b2) ; `shared/utils/`, un module neuf et son test — et `tests/e2e/backup-schedule.spec.ts` ; plus
la constante de la garde de comptage `shared/i18n-keys.test.ts`, comptée avec l'i18n, AC-b8),
`kesh-i18n` (catalogues, quatre locales : deux clés neuves, une modifiée), manuel d'administration FR (`.tex` et PDF) —
**3** au grain des crates et paquets ; **5** au grain des modules métier de premier niveau que cite `CLAUDE.md`
(`features/admin-backup`, `features/admin-restore`, `shared/utils`, `kesh-i18n`, le manuel). **Aucun fichier Rust** n'est
modifié (`kesh-api` ne l'est plus : `auto_backup.rs`, `routes/backup_schedule.rs`, `i18n_e2e.rs`,
`admin_auto_backup_e2e.rs` et `textes_coherents.rs` sont à la 32-1d ou à la 32-1a). Pas de migration, pas de `kesh-db`.
**Dépend de la 32-1d** (le bandeau lit `lastSuccessInFuture`, que la 32-1d sert) et de la **32-1c** (le PDF) : fusionnée
après les deux (Dev Notes, § *Pourquoi quatre clés*).

### Critères d'acceptation

1. **AC-b1 — E-6 : une saisie non enregistrée survit à « Sauvegarder maintenant », sans défaire le verrou
   optimiste** (C-32-1-8, amendé par C-32-1-29). `AutoBackupSection.svelte` : un état neuf **`formBase`** — le réglage
   (`BackupSchedule`) sur lequel la saisie a été faite. `apply(value)` (gardé par `loadSchedule` et `save`) pose
   `schedule`, l'état partagé, les trois champs **et** `formBase = value`. `applyState(value)` (employé par `runNow` et par
   la relecture de l'AC-b2) pose `schedule` et l'état partagé ; si les trois champs **égalent** encore ceux de
   `formBase` (saisie non modifiée), il les remet à `value` et pose `formBase = value` ; sinon il ne touche **ni** les
   champs **ni** `formBase`. `save` envoie **`formBase.version`** (non plus `schedule.version`) : une saisie faite sur
   la version *n* s'enregistre contre *n*. Dans le cas normal la réponse d'un run porte la même `version` (`record_*` ne
   la touche jamais), et rien ne change ; si un autre administrateur a enregistré entre-temps (*n* + 1), l'enregistrement
   rend **409** et l'écran ouvre « Recharger » (comportement existant, `auto-backup-conflict`) au lieu d'écraser son
   réglage en silence. Vitest neufs (bloc « Sauvegarder maintenant ») : (i) la fréquence passée de 24 à 12 **sans
   enregistrer**, puis run réussi (`runBackupNow` rend `lastStatus: 'ok'`, `intervalHours: 24`, même `version`) → le
   sélecteur montre **12**, l'état affiché est celui de la réponse (dernière tentative réussie), et « Enregistrer »
   envoie `intervalHours: 12` avec la `version` lue au montage ; (ii) sans saisie, après un run dont la réponse porte
   `retentionCount: 9`, le champ montre 9 ; (iii) **concurrence** — saisie modifiée, puis run dont la réponse porte
   `version + 1` et `retentionCount: 10` → le formulaire garde la saisie, « Enregistrer » envoie la `version` **lue au
   montage**, et `saveBackupSchedule` rejeté en 409 `OPTIMISTIC_LOCK_CONFLICT` → `auto-backup-conflict` présent.
   **Rouge constaté avant le correctif** (le cas (i) montre 24). **Mutations** : **M-b1** — `applyState` qui réécrit
   toujours le formulaire → (i) rouge ; **M-b9** — `save` qui envoie `schedule.version` → (iii) rouge (la version
   envoyée est *n* + 1).
2. **AC-b2 — E-7 : une réponse qui manque relit l'état** (C-32-1-9, amendé par C-32-1-22, C-32-1-30 et C-32-1-34).
   Dans `runNow`, l'échec se classe en trois cas, et le message « pas parvenue » est **réservé** au premier :
   - **issue inconnue** — (a) `isApiError(err)` et `code === 'TIMEOUT'`, `code === 'NETWORK_ERROR'` (tous deux
     `status: 0`, `api-client.ts:312-321`), `status === 502` ou `status === 504` : le prédicat **même** d'`issueInconnue`
     (`features/admin-restore/AdminRestorePanel.svelte:61-68`), à **extraire** dans un module partagé
     (`frontend/src/lib/shared/utils/`, nom au développement) et employé par les deux écrans (règle DRY ; aucun
     comportement de l'import ne change, le prédicat extrait restant faux pour une erreur qui n'est pas une
     `ApiError`). Le module neuf importe `isApiError` de **`./api-error`**, non de `./api-client` — à la différence
     d'`AdminRestorePanel.svelte:20`, qui l'importe aujourd'hui d'`api-client` et n'est pas le modèle à suivre ici : l'en-tête d'`api-error.ts` (l. 5-9) dit que dix-huit fichiers de test
     simulent `$lib/shared/utils/api-client` par une fabrique qui ne fournit presque jamais `isApiError`, si bien
     qu'un écran qui emploierait le prédicat casserait sous ces tests ; **ou** (b) un rejet qui **n'est pas** une `ApiError` — le cas réel : un 200 dont le corps ne se lit pas,
     que `request()` rend `undefined` (`api-client.ts:480-484`, `catch { return undefined as T; }`), si bien qu'`applyState(undefined)` lève ; la tentative,
     elle, est terminée (l'archive écrite si elle a réussi ; voir la limite ci-dessous). `runNow` teste donc `!isApiError(err) || <prédicat>(err)` → `runError` =
     `admin-backup-auto-run-unknown` (« La réponse du serveur n'est pas parvenue : la sauvegarde a peut-être été écrite,
     ou s'écrit encore. L'écran relit l'état et la liste ci-dessous ; une sauvegarde encore en cours n'y paraît pas
     terminée : rechargez la page dans quelques minutes. », valeur fr-CH à reprendre telle quelle ou à affiner, quatre
     locales — elle ne dit **pas** que la relecture a réussi, ni que l'état relu est le résultat), puis relecture de
     l'état et de la liste. **Limite assumée, cas (b)** (validation P3, F10 ; C-32-1-38) : pour le 200 au corps
     illisible, la réponse **est** parvenue et la sauvegarde est **terminée** — réussie ou en échec : le 200 rend
     l'état *après* la tentative, quel que soit son statut (`routes/backup_schedule.rs:144-146`) — ; la relecture montre
     alors son résultat, et la première moitié du message (« pas parvenue ») comme « ou s'écrit
     encore » y sont inexactes ; le message ne reste vrai que par sa disjonction (« a peut-être été écrite »). C'est le
     prix d'un message unique pour un cas rare (un corps de 200 abîmé en route) ; une variante propre (une clé de plus)
     n'est pas prescrite. Le message n'est donc **pas** « vrai dans chaque cas » : il l'est pour TIMEOUT, NETWORK_ERROR,
     502 et 504. **Pourquoi « s'écrit encore »** (C-32-1-30) : un proxy coupe avant la fin (502/504) pendant
     que la tâche `tokio::spawn` continue (`routes/backup_schedule.rs:152`) ; la relecture tombe alors entre
     `record_attempt` (qui ne pose que `last_attempt_at`, `repositories/backup_schedule.rs:250-258`) et
     `record_success`/`record_failure` : le panneau montre la date de la tentative en cours avec le statut de la
     **précédente** (« réussie » si elle l'était, `AutoBackupSection.svelte:339-343`), et la liste n'a pas encore le
     fichier. Rien dans l'état servi ne distingue « en cours » de « terminée en échec » (`record_failure` ne pose aucune
     date) : l'écran ne peut pas le dire sans un champ neuf — hors périmètre, versé en commentaire de [#725], dont la
     partie 1 prévoit déjà un signal « tentative en cours » ;
   - **autre réponse 5xx** (500 d'E-2 compris, 503) : le serveur a répondu, et l'archive a pu être écrite (E-2) →
     `runError` = le message du serveur (`errorText`, repli `admin-backup-auto-run-error` actuel), **et** relecture de
     l'état et de la liste ;
   - **4xx** (401, 403, 409 `BACKUP_IN_PROGRESS`, 423, 429…) : le serveur a refusé, rien n'a été écrit → message du
     serveur (`errorText`, comportement actuel), **aucune** relecture.
   **Relecture** : l'état par `fetchBackupSchedule` → `applyState` (non par `loadSchedule`, qui remet `conflict` et
   `saveError` et réécrit le formulaire) ; la liste par `loadBackups`. Un échec de la relecture de l'état pose
   `loadError` (même texte que `loadSchedule`, `AutoBackupSection.svelte:78-84` : alerte `auto-backup-load-error`
   au-dessus du panneau, le réglage affiché restant le précédent) ; un échec de la liste pose `listError` (déjà fait par
   `loadBackups`). **`loadError` remis à `null`** au départ de `runNow`, comme `runError` (l. 142) : sans quoi une alerte
   posée par une relecture en échec resterait au-dessus d'un état relu ensuite avec succès — seul `loadSchedule` l'efface
   aujourd'hui (l. 72) ; l'alerte d'un premier chargement en échec n'est pas concernée (`schedule` nul, le panneau et son
   bouton ne s'affichent pas). Vitest neufs : `{ code: 'TIMEOUT', status: 0 }` → message neuf, `fetchBackupSchedule`
   appelé une fois de plus qu'au montage, `fetchServerBackups` aussi ; idem `{ status: 504 }` ; `runBackupNow` résolu en
   `undefined`, et `mockRejectedValue(new TypeError('x'))` → message neuf, relecture faite ; `TIMEOUT` puis une relecture
   qui rend `lastAttemptAt` neuf et `lastStatus: 'ok'` ancien (la tentative en cours) → message neuf affiché, qui dit
   « s'écrit encore » ;
   `{ status: 500, message: 'Erreur interne' }` → « Erreur interne », relecture faite ; `{ status: 403, message: 'Accès
   refusé' }` → « Accès refusé », **aucune** relecture ; `TIMEOUT` puis `fetchBackupSchedule` rejeté → message neuf
   **et** `auto-backup-load-error` présent, puis un second « Sauvegarder maintenant » réussi → `auto-backup-load-error`
   **absent** ; le test 409 existant (`AutoBackupSection.test.ts:210`) reste vert **et** assert qu'aucune relecture
   d'état n'a lieu ; les tests d'`AdminRestorePanel` restent verts (prédicat extrait). **Rouges constatés avant le
   correctif** (TIMEOUT, 504, 500, `undefined`). **Mutations** : **M-b2** — retirer la relecture → rouge ; **M-b8** —
   étendre le message neuf aux 4xx (prédicat `status !== 409`) → le cas 403 rouge ; **M-b10** — retirer
   `loadError = null` du départ de `runNow` → le cas « second run réussi » rouge.
3. **AC-b3 — E-8 : un échec de sondage n'éteint plus le bandeau ; une réponse tardive est ignorée.**
   `backup-schedule.svelte.ts` : un échec de lecture **laisse** `_value` ; un compteur de séquence, incrémenté par
   `setBackupScheduleState` et au départ de chaque `refreshBackupScheduleState`, fait ignorer toute réponse dont la
   séquence n'est plus la dernière (C-32-1-10). Commentaires mis d'accord, **quatre** sites : le doc-comment du module
   `backup-schedule.svelte.ts`, celui du getter (l. 14 : « `null` tant qu'il n'est pas lu, ou si la lecture a échoué »
   — la seconde moitié devient fausse), l'en-tête de `BackupScheduleBanner.svelte` (l. 10-12) et celui de
   `BackupScheduleBanner.test.ts` (l. 1-3 : « jamais si la lecture échoue » — vrai seulement d'une première lecture). Vitest neufs (`BackupScheduleBanner.test.ts`) : (i) état `failed` affiché, sondage suivant en échec →
   bandeau **toujours** `failed` ; (ii) une lecture lancée, puis `setBackupScheduleState(ok)`, puis la lecture rend
   `failed` → **aucun** bandeau ; (iii) le test existant « lecture en échec : absent, sans erreur » (l. 106) reste vert.
   **Rouges constatés avant le correctif.** **Mutations** : **M-b3** — rétablir `_value = null` dans le `catch` →
   (i) rouge ; **M-b4** — ignorer la séquence → (ii) rouge.
4. **AC-b4 — L-B5, le bandeau : une réussite datée du futur se dit telle.** *Le drapeau servi
   (`is_success_in_future`, champ `lastSuccessInFuture` de `BackupScheduleResponse`, test `prochaine_et_retard`) est
   l'AC-d1 de la 32-1d, qui fusionne avant ; cet AC en est le lecteur (C-32-1-37).*
   - `BackupSchedule` (frontend, `backup-schedule.api.ts:34-51`) porte `lastSuccessInFuture: boolean`, **requis** (le
     serveur le sert toujours une fois la 32-1d fusionnée ; C-32-1-28). Fixtures typées à compléter de
     `lastSuccessInFuture: false`, sans quoi `npm run check` rougit : `backup-schedule.api.test.ts:44` (`schedule()`),
     `AutoBackupSection.test.ts:45`, `BackupScheduleBanner.test.ts:43` ; et, non typée mais à tenir fidèle à la réponse
     servie, la fonction `etat()` de `frontend/tests/e2e/backup-schedule.spec.ts:111-125`.
   - `BackupScheduleBanner.svelte` : `kind === 'overdue'` **et** `lastSuccessInFuture` → clé neuve
     `admin-backup-banner-future` (« La dernière sauvegarde automatique réussie est datée du { $date }, après l'heure du
     serveur : son horloge a reculé. », quatre locales) ; sinon le texte actuel. `bannerKind` inchangé. Vitest neuf
     (`BackupScheduleBanner.test.ts`) : état `overdue` avec `lastSuccessInFuture: true` → le texte neuf, la date
     formatée ; même état avec `false` → le texte actuel. **Rouge constaté avant le correctif.**
   **Mutation M-b11** (ajoutée au découpage, C-32-1-37 : la M-b5 d'origine, côté serveur, est devenue la M-d1 de la
   32-1d, et le bandeau n'avait aucune mutation propre) : le bandeau qui ignore `lastSuccessInFuture` → le Vitest neuf
   rouge (cas `true`).
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
6. **AC-b6 — déplacé** : L-B7 (le refus du réglage traduit) est l'**AC-d2** de la 32-1d (C-32-1-37).
7. **AC-b7 — L-B3 et les autres phrases du manuel.** `docs/manual/fr/admin-manual.tex`, § *Sauvegarde automatique* :
   - *Espace disque requis* (l. 2349) : le seuil **ne mesure pas la base** ; il ne la compte qu'à travers la dernière
     sauvegarde réussie, qui la contient — et seulement quand celle-ci est **plus grande que les documents** :
     `seuil_d_espace` (l. 496-500) part de `max(dernière, documents)` (validation P3, R5 ; C-32-1-38). **Quand la
     dernière sauvegarde réussie n'est pas plus grande que les documents — en particulier avant la première**
     (`last_success_bytes` absent, compté 0 : `auto_backup.rs:680-683`) —, le seuil vaut 1,2 fois le total des documents
     plus 256 Mio : l'archive contenant la base **et** les documents, la base n'y est couverte que par la marge — un
     cinquième de la taille des documents, plus 256 Mio —, **quelle que soit** sa taille rapportée aux documents (C-32-1-33 ;
     la phrase « une base plus grosse que le dossier des documents n'est couverte que par 256 Mio », prescrite en P1,
     était fausse dans les deux sens). Phrase à écrire, dans ce sens : « Le seuil ne mesure pas la base : il ne la compte
     qu'à travers la dernière sauvegarde réussie, qui la contient, quand celle-ci est plus grande que les documents.
     Sinon — en particulier avant la première sauvegarde réussie —, la base n'y est couverte que par la marge — un
     cinquième de la taille des documents, plus 256 Mio — : si la base dépasse cette marge, le disque peut se remplir
     pendant l'écriture (le cas que le paragraphe décrit ensuite : fichier partiel supprimé, sauvegardes existantes
     gardées). » (Placement et liaison avec la phrase du disque rempli au développement ; le
     contrôle aplati porte sur « un cinquième de la taille des documents ».) La **dernière phrase** du paragraphe (« Sur un NAS, la base partage souvent le volume du dossier de
     sauvegarde : ce seuil protège aussi la base. ») est réécrite pour ne plus contredire la précédente : le seuil ne
     mesure pas la base, mais en refusant d'écrire il évite de remplir le volume qu'elle partage ;
   - *Ce que l'écran montre* (l. 2345) : une modification du réglage non enregistrée est **conservée** par « Sauvegarder
     maintenant » (et si un autre administrateur a enregistré entre-temps, « Enregistrer » le signale et propose
     « Recharger », AC-b1) ; si la réponse ne parvient pas (délai, coupure, 502, 504) ou si le serveur rend une erreur
     (5xx, 503 compris), l'écran relit l'état et la liste — **sans** écrire « où se lit le résultat » (C-32-1-30) : une
     sauvegarde encore en cours y montre la date de la tentative avec le statut de la **précédente**, et le fichier n'est
     pas encore dans la liste ; l'issue se lit en rechargeant la page une fois l'écriture finie (AC-b2) ;
   - *Le bandeau de l'application* (l. 2347) : un échec de lecture **garde** le dernier état lu ; et la phrase « « Aucune
     sauvegarde automatique réussie depuis le … » quand la dernière réussite date d'au moins deux intervalles » reçoit
     son cas futur (renvoi à *Horloge du serveur déréglée* et au texte de l'AC-b4). Cette phrase porte la locution que
     la garde **G22** exige (`textes_coherents.rs`, `la_sauvegarde_automatique_dit_vrai`, l. 1599-1663 : sur tout
     `admin-manual.tex` normalisé, elle exige « au moins deux intervalles » et « retentée au bout d'une heure. », et
     refuse « plus de deux intervalles » et « retentée … plus court ») : la locution « au moins deux intervalles » reste, et
     la phrase du cas futur n'emploie aucune tournure « plus de deux intervalles » (validation P3, F9) ;
   - *Horloge du serveur déréglée* (l. 2357) : le bandeau dit que la dernière réussite est datée **après** l'heure du
     serveur (texte de l'AC-b4), au lieu de « aucune sauvegarde réussie depuis le … » ;
   - *Les sauvegardes du serveur* (l. 2355) : la mention citée dans sa forme neuve (AC-b5).
   `make admin` sous verrou (T-b6) ; **contrôle aplati** du PDF (Dev Notes, § *Contrôle aplati d'un PDF* : apostrophe
   typographique normalisée, témoin positif d'abord) pour chaque phrase neuve, et `grep -c '??'` sur le texte aplati : 0.
   Les gardes qui lisent le manuel restent vertes (`binary(textes_coherents)` — G22 nommément —,
   `binary(configuration_transmise)`).
8. **AC-b8 — i18n.** Deux clés neuves (`admin-backup-auto-run-unknown`, `admin-backup-banner-future`) et une modifiée
   (`admin-backup-server-too-large`, variable `$size`) dans les quatre catalogues — la troisième clé neuve d'origine,
   `admin-backup-schedule-error-out-of-domain`, est à la 32-1d (AC-d3) ; `parity_between_locales`
   (`kesh-i18n/src/loader.rs`) vert ; G9 vert (inchangé par la 32-1b) ;
   `npm run lint-i18n-ownership` vert (préfixe `admin-backup-` dans `features/admin-backup/`) ; les gardes frontend
   `frontend/src/lib/shared/i18n-entrees-a-variables.test.ts` (variable `$size`, AC-b5) et
   `i18n-repli-divergent-actif.test.ts` (repli Svelte de la clé modifiée égal au fr-CH) vertes. Recompte :
   `grep -c '^admin-backup-' crates/kesh-i18n/locales/*/messages.ftl` aux deux bornes — base de la 32-1b sur `main`
   **après** la fusion de la 32-1d, et sa tête —, **+2** par locale (la 32-1d en ajoute une, comptée à sa propre
   fusion).
   **Garde de comptage des sites** (validation P3, R2) : `frontend/src/lib/shared/i18n-keys.test.ts` assert
   `releve.sitesTotal` égal à `ATTENDU.sitesTotal` (l. 715 : **2332** au 2026-10-24, l. 1025) ; la 32-1b ajoute deux
   sites `i18nMsg(` (`admin-backup-auto-run-unknown` dans `runNow`, `admin-backup-banner-future` dans
   `BackupScheduleBanner.svelte`) : la constante est **relevée de 2** (2332 → 2334 attendu), avec le commentaire daté
   qu'exige le fichier (story, valeurs aux deux bornes, commande `grep -oE '(i18nMsg|[^a-zA-Z]msg)\('` recomptée par
   fichier, périmètre déclaré, comme le paragraphe de la Story 31-1c au-dessus de la l. 715) ; un site de plus si la
   relecture de l'AC-b2 réécrit le texte de `loadError` au lieu de le partager avec `loadSchedule` — le compte se lit à
   la source, il ne se suppose pas. La 32-1d n'ajoute aucun site (clé servie par Rust seule).
9. **AC-b9 — recette visuelle** (thème clair, base de démonstration, `kesh-gate-logs/321-captures/`) : (a) formulaire
   modifié conservé après « Sauvegarder maintenant » ; (b) message de réponse manquante (délai simulé par une route
   interceptée en E2E, ou Vitest à défaut — dire lequel) ; (c) bandeau « datée après l'heure du serveur » (prise après
   la fusion de la 32-1d, sur un serveur qui sert le drapeau, ou par une route interceptée — dire lequel) ; (d) ligne
   « trop grande … soit … ». La capture (e) d'origine, le refus du réglage en allemand, est l'AC-d4 de la 32-1d.
   Jointes au Dev Agent Record et au prompt de revue.
10. **AC-b10 — E2E.** `frontend/tests/e2e/backup-schedule.spec.ts`, test « Administrateur : réglage gardé… » (l. 139) :
    avant « Sauvegarder maintenant », modifier le nombre conservé **sans enregistrer** ; après le 200, le champ garde la
    valeur saisie (AC-b1 à travers la frontière HTTP). Gate **complet**, les quatre gates frontend, **E2E complet au
    dernier commit de code**, jugé fichier par fichier contre `docs/testing.md` § *Les échecs attendus* — joué sur la
    branche **mise à jour sur `main` après la fusion de la 32-1d et de la 32-1c** (le backend servi porte alors
    `lastSuccessInFuture`) : un E2E joué avant cette mise à jour ne compte pas pour le dernier commit de code.
11. **AC-b11 — mutations** M-b1 à M-b4 et M-b7 à M-b11 (neuf ; M-b7 en deux formes ; M-b9 et M-b10 de la validation
    P2 ; M-b11 du découpage ; même protocole que l'AC-a13). **M-b5** et **M-b6** sont parties avec la 32-1d (**M-d1**,
    **M-d2**) ; leurs numéros ne sont pas réemployés.

### Tâches

- [ ] **T-b0** — Base et port propres (comme T-a0) ; dossiers E2E `/tmp/kesh-e2e-321/…` passés au backend **et** au runner.
- [ ] **T-b1** — *déplacée* : le refus traduit est la T-d1 de la 32-1d.
- [ ] **T-b2 (AC-b4)** — Vitest rouge du bandeau, type `BackupSchedule` et fixtures, bandeau, clé × 4 ; M-b11. Sur la
  branche mise à jour après la fusion de la 32-1d.
- [ ] **T-b3 (AC-b1, AC-b2)** — Vitest rouges, `applyState`, prédicat d'issue inconnue extrait (AdminRestorePanel inchangé en
  comportement), `formBase`, `runNow` en trois cas (non-`ApiError` comprise), `loadError` remis à `null`, clé × 4 ;
  M-b1, M-b2, M-b8, M-b9, M-b10.
- [ ] **T-b4 (AC-b3)** — Vitest rouges, séquence, doc-comments.
- [ ] **T-b5 (AC-b5)** — clé modifiée × 4, repli Svelte, test ; M-b7 (deux formes).
- [ ] **T-b6 (AC-b7)** — manuel, **`make admin`** sous verrou (non `make fr` : après le rebase sur la 32-1c, les dates
  posées par le checkout peuvent faire reconstruire le manuel utilisateur et la brochure, des diffs binaires que la 32-1b
  ne porte pas ; seul `admin-manual.pdf` est commité), contrôle aplati ; après fusion de la 32-1c, mise à jour sur
  `main` et **régénération** du PDF (C-32-1-17).
- [ ] **T-b7 (AC-b9, AC-b10, AC-b11)** — après la fusion de la 32-1c **et** de la 32-1d : mise à jour sur `main`,
  E2E, captures, mutations, gates ; Dev Agent Record chiffré.

---

## C — 32-1c : la procédure de répétition (#714, hors ses deux choix tranchés)

**Modules** : manuels FR (`docs/manual/fr/admin-manual.tex`, `docs/manual/fr/user-manual.tex` pour ⚠️ seulement,
`docs/manual/shared/kesh-style.sty` pour la macro, les deux replis d'icônes et le `literate` des listings (C-32-1-31), et
les **trois** PDF — la brochure se régénère aussi, sa règle du `Makefile` dépendant du `.sty`, C-32-1-26 ; son `.tex`
ne change pas) ; fiche `31-4-repetition-a-blanc.md` (annotations
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
7. **AC-c7 — ⚠️ imprimé, et aucun autre signe sans glyphe** (P2 B2-L5, C-32-1-16, C-32-1-31). Macro `\keshAttention` dans `docs/manual/shared/kesh-style.sty`,
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
   **La classe entière, par son symptôme** (C-32-1-31, validation P2 F3 = R2) — les 25 autres caractères sans glyphe des
   trois PDF, même cause et même fichier (`kesh-style.sty`), traités par le même geste, sans toucher aucun `.tex` :
   - **replis d'icônes** : `\providecommand{\faCheckCircle}{\textbf{+}}` (l. 39, au lieu de `✓`) et
     `\providecommand{\faExchangeAlt}{\textbf{→}}` (l. 42, au lieu de `⇄` ; `→` est déjà le repli de `\faRocket` et
     s'imprime) — signes que TeX Gyre Heros possède ; les 16 sites de la brochure ;
   - **listings** : au style `kesh` (`\lstdefinestyle{kesh}`, l. 221-242), `literate={✓}{{+}}1 {✗}{{x}}1 {═}{{=}}1`
     — signes que TeX Gyre Cursor possède ; les 9 sites des deux scripts d'exemple du manuel d'administration (l. 1547,
     3313, 3318, 3320), hors `sec:repetition-cloture` (aucun de ses listings ne porte ces caractères : son rendu ne
     change pas, et son `git diff` reste vide, AC-c9). La source des listings garde ses caractères ; le PDF montre le
     repli.
   Le choix des signes de repli est de présentation et **réversible** (une ligne du `.sty` chacun). **Contrôle
   décidable, aux deux bornes** : `pdftotext docs/manual/fr/<pdf> - | grep -o $'\xef\xbf\xbd' | wc -l` rend
   **26 → 0** (administration), **9 → 0** (utilisateur), **16 → 0** (brochure), écrits au Dev Agent Record ; témoins
   aplatis : « + Conforme au Code des obligations suisse » (brochure), « echo "+ Sauvegarde terminee » (administration,
   listing ; aplati sans `tr -s` si l'espacement du listing le demande). Le `grep -c "⚠"` sur un PDF, qui rend 0 avant
   comme après, **n'est pas** un contrôle et n'est pas écrit comme tel. Si `literate` ne prend pas un caractère multioctet
   sous XeLaTeX (le compte de U+FFFD le dit), le repli est d'écrire ces 9 signes en ASCII dans les deux scripts
   d'exemple (déjà sans accents : « terminee », « integrite »), et de le dire au Dev Agent Record.
8. **AC-c8 — la fiche de la 31-4 annotée, non réécrite** (P2 A-LOW-4, P4 LOW-4). Deux annotations datées
   (« *Annotation du 2026-10-2x, Story 32-1c* ») : à l'AC11 (`31-4-repetition-a-blanc.md:537-543`), G15 ne lit rien de la
   sous-section (elle écrit ses volumes en `\keshpath{…}`, G15 lit `\texttt{…}` et `/var/lib/docker/volumes/`) ; à la
   l. 504, le `test ! -e "$COPIE"` décrit n'est plus le contrôle final (renvoi à l'annotation de la l. 435). Le texte
   d'origine reste.
9. **AC-c9 — le manuel et ses gardes.** `make fr` sous verrou ; chaque phrase neuve des AC-c1 à AC-c5 contrôlée au PDF
   aplati (Dev Notes, § *Contrôle aplati d'un PDF*) ; la brochure régénérée garde son texte **hors ses 16 signes**
   (C-32-1-26, amendé par C-32-1-31) : la version d'avant est extraite dans `kesh-gate-logs/` depuis **la base de la
   branche**, `git show "$(git merge-base origin/main HEAD)":docs/manual/fr/marketing-brochure.pdf` — **jamais**
   `HEAD:`, qui, après le commit du PDF régénéré, désigne la version d'après et rendrait le contrôle vert par
   construction —, puis les deux `pdftotext` sont aplatis (`tr '\n' ' '`), privés de U+FFFD, de `+` et de `→`, et
   comprimés (`tr -s ' '`) : textes **identiques**. Commande exacte (validation P3, F5), lancée depuis la racine du
   worktree — le retrait des caractères multioctets passe par `sed` avec les caractères **entiers**, écrits en
   `$'…'` (un `tr -d '→'` agirait par octets et abîmerait d'autres caractères qui partagent l'octet E2, `’` compris) :
   ```sh
   git show "$(git merge-base origin/main HEAD)":docs/manual/fr/marketing-brochure.pdf \
     > /home/gcorbaz/devel/kesh-gate-logs/321-brochure-avant.pdf
   aplatir() { pdftotext "$1" - | tr '\n' ' ' | sed $'s/\xef\xbf\xbd//g; s/+//g; s/\xe2\x86\x92//g' | tr -s ' '; }
   cmp <(aplatir /home/gcorbaz/devel/kesh-gate-logs/321-brochure-avant.pdf) \
       <(aplatir docs/manual/fr/marketing-brochure.pdf) && echo identiques
   ```
   (témoin positif d'abord : `aplatir` sur le PDF d'avant rend un texte non vide qui contient « Conforme au Code des
   obligations suisse ») ; référence employée et sortie de `cmp` écrites au Dev Agent Record ; gate ciblé `binary(textes_coherents)` et
   `binary(configuration_transmise)` (gardes G14 à G16, G19, qui lisent la sous-section) — C-32-1-18. Aucune commande de
   la sous-section ne change : `git diff` des `lstlisting` de `sec:repetition-cloture` **vide** (contrôle écrit au Dev
   Agent Record).
10. **AC-c10 — #714 sans constat propre.** Après la 32-1c, chaque ligne de #714 est soit corrigée (AC-c1 à AC-c8), soit
    dans [#726] (*Issues ouvertes*, n° 2), soit écartée avec sa raison (la moitié fausse de P3 E L-3).

### Tâches

- [ ] **T-c1 (AC-c1 à AC-c5)** — phrases ; **aucune** commande touchée.
- [ ] **T-c2 (AC-c6, AC-c7)** — `\needspace`, macro, 26 remplacements (`⚠️` entier), deux replis d'icônes, `literate`
  des listings ; U+FFFD compté aux deux bornes dans les trois PDF (26, 9, 16 → 0) ; captures.
- [ ] **T-c3 (AC-c8)** — deux annotations datées.
- [ ] **T-c4 (AC-c9)** — `make fr`, contrôles aplatis, gate ciblé ; Dev Agent Record.

---

## D — 32-1d : le serveur de la sauvegarde automatique (#711 : L-B5 (drapeau servi), L-B7)

*Née du découpage de la 32-1b avant la validation P3 (C-32-1-37). Ses AC reprennent, sans en changer le fond, la moitié
serveur de l'ancien AC-b4 (AC-d1) et l'ancien AC-b6 (AC-d2) ; ses mutations sont les anciennes M-b5 (M-d1) et M-b6
(M-d2).*

**Modules** : `kesh-api` (`auto_backup.rs` — `is_success_in_future` près d'`is_overdue` et ses tests dans le `mod tests` ;
`routes/backup_schedule.rs` — le champ servi et le refus traduit ; tests `admin_auto_backup_e2e.rs`, `i18n_e2e.rs`,
`textes_coherents.rs`), `kesh-i18n` (une clé neuve, quatre locales) — **2** au grain des crates ; **3** au grain des
modules métier (`kesh-api/auto_backup`, `kesh-api/routes/backup_schedule`, `kesh-i18n`). **Aucun fichier du frontend ni du
manuel** : un champ ajouté à la réponse est ignoré par le client actuel (le type `BackupSchedule` ne le déclare pas
encore ; `npm run check` reste vert), et le refus, déjà affiché tel que le serveur le rend (`errorText`), n'a pas de
phrase au manuel (`grep -n 'hors domaine' docs/manual/fr/*.tex` : rien). Pas de migration, pas de `kesh-db`, aucune
table : le `.keshbackup` ne change pas. **Fusionnée avant la 32-1b**, qui lit son drapeau.

### Critères d'acceptation

1. **AC-d1 — L-B5, le drapeau servi : une réussite datée du futur se dit telle** (ex-AC-b4, moitié serveur).
   - `auto_backup.rs` : `pub fn is_success_in_future(schedule: &BackupSchedule, now: NaiveDateTime) -> bool`, vrai si
     la sauvegarde est **activée**, que `last_success_at` existe et que `passe()` l'écarte (l. 183) — la garde
     `enabled` calquée sur `is_overdue` (l. 221-226), pour que `lastSuccessInFuture` vrai implique `overdue` vrai et que
     l'API ne serve jamais deux drapeaux contradictoires (C-32-1-24) ; tests unitaires à côté de
     `is_overdue_dans_le_futur` (l. 950) : futur → vrai, passé → faux, absent → faux, égal à `now` → faux, **futur et
     désactivée → faux**.
   - `BackupScheduleResponse` (`routes/backup_schedule.rs:55-74`, `new` l. 76-95) : champ `last_success_in_future: bool`
     (`lastSuccessInFuture`), calculé par `new`, toujours servi.
   - `prochaine_et_retard` (`admin_auto_backup_e2e.rs:1121`) assert `lastSuccessInFuture` **faux** dans les cas passés et
     **vrai** dans le cas futur (assertion `overdue` de la l. 1159) — **rouge avant** (`null`).
   **Mutation M-d1** (ex-M-b5) : `is_success_in_future` qui rend toujours `false` → le test e2e rouge.
2. **AC-d2 — L-B7 : le refus du réglage traduit** (ex-AC-b6). `update_backup_schedule` : `AppError::Validation(t(
   "admin-backup-schedule-error-out-of-domain", "<valeur fr-CH exacte>"))` ; clé dans les quatre catalogues ; entrée
   ajoutée à la table de G9 (`textes_coherents.rs`, `les_replis_rust_suivent_le_catalogue`, `TABLE: [(&str, Cles); 4]` →
   `5`, fichier `crates/kesh-api/src/routes/backup_schedule.rs`, un site, sans paramètre). Test neuf dans
   `crates/kesh-api/tests/i18n_e2e.rs` (C-32-1-13) : application montée en `DeCh` (`spawn_app_with_locale`), connexion
   administrateur, `GET` du réglage pour lire `version`, `PUT /api/v1/admin/backup-schedule` avec `intervalHours: 7`
   → **400**, `VALIDATION_ERROR`, message **égal** au rendu `de-CH` de la clé et **différent** du rendu `fr-CH` (marques
   d'isolation retirées, comme `audit_log_e2e.rs:396-405`). **Rouge constaté avant le correctif** (message français).
   Le test existant du `PUT` hors domaine (`admin_auto_backup_e2e.rs:849`) n'assert que le statut : il reste vert.
   **Mutation M-d2** (ex-M-b6) : rétablir la chaîne française → rouge.
3. **AC-d3 — i18n.** Une clé neuve (`admin-backup-schedule-error-out-of-domain`) dans les quatre catalogues ;
   `parity_between_locales` vert ; G9 vert (table à **5**) ; `npm run lint-i18n-ownership` et les gardes frontend
   `i18n-entrees-a-variables.test.ts`, `i18n-repli-divergent-actif.test.ts` vertes (les catalogues changent, aucun site
   Svelte). Recompte : `grep -c '^admin-backup-' crates/kesh-i18n/locales/*/messages.ftl` aux deux bornes (base de la
   branche, tête), **+1** par locale.
4. **AC-d4 — recette visuelle** (ex-AC-b9 (e) ; thème clair, base de démonstration, `kesh-gate-logs/321-captures/`) :
   le refus du réglage en allemand, si l'écran permet de le provoquer — le sélecteur de fréquence et le champ du nombre
   conservé bornent la saisie, si bien qu'il faut sans doute une requête forgée ou une route interceptée ; sinon dire
   pourquoi au Dev Agent Record. Le drapeau `lastSuccessInFuture` n'a aucun rendu dans la 32-1d (capture (c) de la
   32-1b).
5. **AC-d5 — gates.** Gate backend **complet** (`scripts/test-fast.sh`, base remise à zéro) avant le push ; en boucle de
   revue, gate ciblé `binary(admin_auto_backup_e2e)`, `binary(i18n_e2e)`, `binary(textes_coherents)`, les tests du
   `mod tests` d'`auto_backup.rs` et ceux de `kesh-i18n` (`cargo nextest run -p kesh-i18n` : la parité des catalogues,
   `parity_between_locales`, qu'exige l'AC-d3 — sans eux, une clé oubliée dans une locale ne rougirait qu'au gate
   complet ; validation P3, F7) (pas de `kesh-db` : le ciblage est permis) ; les quatre gates frontend (les catalogues
   changent) ; **E2E complet au dernier commit de code** — la réponse servie change, et `backup-schedule.spec.ts` tourne
   contre elle —, jugé fichier par fichier contre `docs/testing.md` § *Les échecs attendus*.
6. **AC-d6 — mutations** M-d1 et M-d2 (deux ; même protocole que l'AC-a13).

### Tâches

- [ ] **T-d0** — Base et port propres (comme T-a0) ; dossiers E2E `/tmp/kesh-e2e-321/…` passés au backend **et** au runner.
- [ ] **T-d1 (AC-d2, AC-d3)** — clé × 4, test `i18n_e2e.rs` **rouge**, `t(…)`, G9 (ex-T-b1).
- [ ] **T-d2 (AC-d1)** — tests rouges (unitaires, e2e), fonction, champ (ex-T-b2, moitié serveur).
- [ ] **T-d3 (AC-d4, AC-d5, AC-d6)** — capture, mutations, gates, E2E ; Dev Agent Record chiffré.

---

## Dev Notes

### Pourquoi quatre clés, et dans quel ordre

*Trois clés jusqu'au découpage de la 32-1b avant la validation P3 (C-32-1-37), qui en a sorti la 32-1d.*

Les quatre parties se développent **séparément**, mais elles ne sont pas sans fichier commun (C-32-1-27, amendé par
C-32-1-37) :
- **32-1a et 32-1d** partagent trois fichiers de code, en hunks distincts : `crates/kesh-api/src/auto_backup.rs` (A : les
  doc-comments des helpers lexicaux et de `gardes_possedes` du `mod tests`, l. 1117-1201 — commentaires seuls depuis le
  retrait de l'AC-a11, C-32-1-38 ; D :
  `is_success_in_future` près de `is_overdue`, l. 221, et ses tests près de `is_overdue_dans_le_futur`, l. 950, dans le
  même `mod tests`), `crates/kesh-api/tests/admin_auto_backup_e2e.rs` (A : `TestEnv` l. 55-129 et
  `dossier_auto_inutilisable` l. 684 ; D : `prochaine_et_retard` l. 1121) et `crates/kesh-api/tests/textes_coherents.rs`
  (A : `DATE_ISO` l. 277 et un test neuf ; D : `TABLE` de G9, l. 694). Fusion textuelle sans conflit attendue ; la clé
  fusionnée **en second** se met à jour sur `main` (rebase), vérifie `git diff` des trois fichiers, et repasse son gate
  **complet** avant sa fusion. *(Ces trois fichiers étaient partagés par la 32-1a et la 32-1b avant le découpage ; la
  32-1b n'en touche plus aucun.)*
- **32-1d et 32-1b** partagent les quatre `crates/kesh-i18n/locales/*/messages.ftl` (clés distinctes : une pour la D,
  deux neuves et une modifiée pour la B) ; et la 32-1b **lit** ce que la 32-1d sert (`lastSuccessInFuture`, AC-b4 ←
  AC-d1) : **32-1d avant 32-1b**. La 32-1b peut se développer en parallèle (ses Vitest simulent la réponse), mais elle
  se met à jour sur `main` après la fusion de la 32-1d, et son E2E complet, sa capture (c), ses mutations et son gate
  complet se jouent **après** cette mise à jour (AC-b10, T-b7).
- **32-1b et 32-1c** partagent `admin-manual.tex` (sections distinctes) et son PDF (binaire : conflit à toute fusion
  parallèle) : **32-1c avant 32-1b**, la 32-1b régénérant le PDF sur le `.tex` fusionné (C-32-1-17). La 32-1d ne touche
  pas le manuel : aucun ordre de plus sur le PDF.
- **32-1a et 32-1b** ne partagent plus aucun fichier (la 32-1a touche `fiscal-years-page.test.ts`, non
  `features/admin-backup/`).
- **32-1b, 32-1c et 32-1d** partagent `CHANGELOG.md` (validation P3, F4) : **un seul** bloc `## [0.16.0] — Non publié`,
  créé par la première des 32-1c et 32-1d qui fusionne (*Fichiers touchés*, CHANGELOG). Les deux étant fusionnables
  dans l'ordre qu'on veut et développées en parallèle, chacune écrit le bloc sur sa branche : **le conflit textuel est
  prévu** au rebase de la seconde, qui le résout en gardant le bloc de `main` et en y ajoutant sa seule ligne sous
  `### Corrigé` (jamais un second bloc) ; la 32-1b, fusionnée en dernier, fait de même. La 32-1a n'y écrit rien.
- **Ordre de fusion** : 32-1a quand elle est prête ; **32-1c** et **32-1d** dans l'ordre où elles sont prêtes ; **32-1b
  en dernier des trois clés B, C, D**. Contraintes, et seulement elles : 32-1c → 32-1b ; 32-1d → 32-1b ; la seconde des
  32-1a et 32-1d à fusionner se rebase et repasse son gate complet (fichiers communs).
- **Les quatre** écrivent dans cette fiche, `sprint-status.yaml` et le registre (F11 de la validation P2) : chacune
  n'écrit que **sa** sous-section du Dev Agent Record (*32-1a*, *32-1b*, *32-1c*, *32-1d*, ci-dessous) et **sa** clé du
  `sprint-status.yaml` ; le Change Log reçoit une entrée par clé, ajoutée en fin ; la suivante à fusionner résout par
  union. Le registre ne reçoit que des entrées neuves, numérotées à la suite au moment de l'écriture.

Mots-clés de PR (à poser par l'orchestrateur, **jamais dans un commit**) : 32-1a `closes #716` ; **`closes #711` porté
par celle des clés qui touchent #711 — 32-1a, 32-1b, 32-1d — qui fusionne EN DERNIER** (E-9 (1) et (2) — et l'angle mort écrit de E-9 (3) — dans la 32-1a, L-B5 et
L-B7 dans la 32-1d, le reste dans la 32-1b ; E-1 et E-2 sont dans [#725], déjà ouverte), les autres portant
`refs #711`. La 32-1d fusionnant toujours avant la 32-1b, elle porte **toujours** `refs #711` ; `closes #711` va donc à
la dernière des 32-1a et 32-1b — à décider au moment d'ouvrir la seconde de ces deux PR, et à vérifier après fusion
(`gh issue view 711 --json state`) ; 32-1c `closes #714` ([#726] étant ouverte). **Depuis la validation P3**
(C-32-1-38), E-9 (3) n'est plus traité par la story : #711 se ferme quand même, chacun de ses constats ayant alors un
sort écrit — corrigé (32-1a, 32-1b, 32-1d), ou suivi ailleurs : E-1 et E-2 par [#725], **E-9 (3) par [#727]**. Le corps
de la PR qui porte `closes #711` le dit en toutes lettres (« E-1 et E-2 suivis par #725 ; E-9 (3) suivi par #727, où
le commentaire de la Story 32-1 le verse »), et le commentaire sur #727 est versé **avant** cette fusion ; sinon, la
PR porte `refs #711` et #711 reste ouverte.

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
- **AC-a10, AC-a11** : l'AC-a11 est retiré (C-32-1-38). Les doc-comments que l'AC-a10 ajoute aux helpers lexicaux
  décrivent leur comportement **actuel**, sans le changer : la 32-1a ne touche ni `crates/kesh-db/tests/lexique/mod.rs`
  ni une ligne de code des helpers, et ne déclare aucun module `lexique` dans `auto_backup.rs`. Le piège qu'avait la
  troisième rédaction (une déclaration `#[cfg(test)]` placée avant le code de production tronquait `code()`, qui
  s'arrête au premier `#[cfg(test)]` : l'assertion `try_lock_owned()` de `gardes_possedes` rougissait, et sa boucle
  des jetons interdits passait à vide sur `auto_backup.rs` — validation P3, R1 = F1) est versé
  au commentaire de [#727], pour que l'extraction ne le rejoue pas.

### 32-1b — remarques de conception

- **AC-b1** : « égalent ceux de `formBase` » se compare sur les trois champs, `Number(intervalHours)` et
  `Number(retentionCount)` (les champs liés aux `<select>`/`<input>` sont des chaînes en Svelte). **Un réglage modifié
  ailleurs** (autre administrateur) pendant la saisie **est** un cas de l'AC (C-32-1-29 ; la note de la validation P1, qui
  le disait hors de l'AC et tenu par le 409, était fausse : `applyState` posait `schedule` — donc sa `version` — sous une
  saisie gardée, et l'enregistrement suivant passait sans 409, écrasant le réglage de l'autre en silence). C'est
  `formBase.version`, non `schedule.version`, qui part avec l'enregistrement : la saisie s'enregistre contre la version
  sur laquelle elle a été faite, et le 409 existant joue.
- **AC-b2** : trois cas (C-32-1-22, C-32-1-34) — issue inconnue (prédicat d'`issueInconnue`, extrait et partagé, **ou**
  rejet qui n'est pas une `ApiError`), autre 5xx, 4xx (409 `BACKUP_IN_PROGRESS` compris) ; seuls les deux premiers
  relisent. La relecture n'appelle pas `loadSchedule` (qui remet `conflict`/`saveError` et réécrit le formulaire) mais
  `fetchBackupSchedule` + `applyState`, et pose `loadError` si elle échoue ; `runNow` le remet à `null` au départ. La
  relecture **ne dit pas le résultat** d'une sauvegarde encore en cours (C-32-1-30) : le message et le manuel le disent.
- **AC-b3** : le motif de séquence est celui de `fiscal-years` (C-29-1-37) ; la séquence est **module-level**, comme
  `_value`.
- **AC-b4** : le bandeau ne compare toujours aucune date (doctrine du module) : il lit le drapeau que la 32-1d sert.

### 32-1d — remarques de conception

- **AC-d1** : `passe()` est privée ; `is_success_in_future` vit dans le même module et la réemploie (une seule règle de
  « futur »).
- **AC-d2** : `i18n_e2e.rs` a son propre verrou (`get_i18n_lock`, l. 27) ; le test neuf le prend, comme ses voisins.
  Les routes d'administration exigent un administrateur **non PAT** (`require_not_pat`) : la connexion par mot de passe
  d'`ensure_admin_user` y suffit. `t()` (`crates/kesh-api/src/errors.rs:36`) rend la clé dans la langue de l'instance.

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
  L-B7 (32-1d depuis le découpage, C-32-1-37) n'en corrige **qu'un** (`backup_schedule.rs`) ; #711 note le « motif répandu dans `routes/` ». Les autres sites
  sont **hors périmètre** et **suivis par [#710]** (ouverte, P4 : « Les refus de validation du backend
  (AppError::Validation) sont écrits en français en dur dans les routes, hors journal d'audit » ; #469, fermée, ne
  couvrait que le journal d'audit). Le développement relève le compte aux deux bornes et l'écrit au Dev Agent Record,
  par un décompte **multiligne** (une forme sur deux lignes — `AppError::Validation(` en fin de ligne, chaîne à la
  suivante — échappe à un `grep` ligne à ligne) et en deux périmètres :
  `grep -rzoP 'AppError::Validation\(\s*"' crates/kesh-api/src/routes | tr '\0' '\n' | grep -c Validation` (chaîne
  littérale ; **145** au 2026-10-24, validation P1) et
  `grep -rzoP 'AppError::Validation\(\s*format!\(\s*"' crates/kesh-api/src/routes | tr '\0' '\n' | grep -c Validation`
  (chaîne formatée ; **101**). Attendu après la 32-1d : le premier baisse de **1**, le second ne bouge pas ; le décompte
  est versé en commentaire sur #710.
- **Mention de taille en Mio ailleurs** : `admin-backup-exceeds-import-limit` (`AdminBackupPanel.svelte:96-99`) dit
  « (N Mio) » sans afficher la taille du fichier à côté : pas de juxtaposition d'unités, hors classe de L-B6.
- **Date ISO** : `2026-02-30` reste une date pour `nombres_isoles` (AC-a7, limite écrite).
- **Gardes lexicales d'`auto_backup.rs`** (E-9 (3), C-32-1-38) : **non corrigées** par la story. `code()` lit comme
  du code les commentaires de fin de ligne, les blocs `/* … */` et le contenu des chaînes ; `argument_de` compte les
  parenthèses des chaînes et des littéraux de caractère ; `corps()` coupe au premier `\n}`, chaîne comprise, et repose
  sur rustfmt. Angle mort assumé, écrit à leurs doc-comments (AC-a10), suivi par [#727]. Constaté le 2026-10-24
  (remédiation P3, `grep`) : les deux jetons exigés par `code()` seul sont du code (`main.rs:383`
  `auto_backup::spawn(` ; `auto_backup.rs:669` `try_lock_owned()`), et les cinq fichiers lus n'ont aucun `/*` hors
  des trois de `lib.rs` (l. 1179 et 1196 dans des `//`, l. 1186 dans une chaîne), que `code()` n'interprète pas ; les
  jetons que `gardes_possedes` cherche par `corps()` et `argument_de` n'ont **pas** été relus ici.
- **Signes sans glyphe dans les PDF** (classe recensée par son symptôme, U+FFFD) : après la 32-1c, **0** dans les trois
  PDF FR (AC-c7) ; aucun autre PDF de manuel n'est versionné (`ls docs/manual/*/*.pdf` : les trois FR seulement).
- **⚠️ hors des manuels FR** : `crates/kesh-report/src/pdf.rs:1532` écrit `⚠` dans un titre de PDF de rapport ; police et
  rendu non vérifiés ici — **hors** (code, autre police) ; à signaler à l'orchestrateur, issue si le glyphe manque au
  rendu.
- **État « tentative en cours » du panneau** (C-32-1-30) : pendant une sauvegarde, le panneau accole à la date de la
  tentative en cours le statut de la **précédente** (`AutoBackupSection.svelte:339-343` ; `record_attempt` ne pose que
  `last_attempt_at`, `record_failure` ne pose aucune date : l'état servi ne distingue pas « en cours » de « terminée en
  échec »). Défaut préexistant, que la relecture de l'AC-b2 rend plus visible ; **hors** — commentaire à verser sur
  [#725], dont la partie 1 prévoit un signal « tentative en cours » (texte ci-dessous, *Commentaires à verser*).
- **E-9 (2)** : angle mort assumé (AC-a10).
- **Copies de l'assistant lexical `code()`** (même `take_while(… != "#[cfg(test)]")` ligne à ligne, sans connaissance
  des blocs `/* … */` ni des chaînes) : `crates/kesh-api/src/admin_backup/export.rs:442` et `:508`,
  `crates/kesh-api/src/routes/admin.rs:1097` (`code_de_production`), `crates/kesh-api/src/document_storage.rs:608`,
  `crates/kesh-api/src/routes/journal_entries.rs:1018` ; `crates/kesh-api/tests/audit_label_registry.rs` a sa propre lecture
  (`strip_line_comments`, l. 179 ; `source_assainie`, l. 231 ; `accolades_hors_chaines`, l. 291). **Complété à la
  validation P2** (R4), par `grep -rn '"#\[cfg(test)\]"\|split("#\[cfg(test)\]")' crates --include=*.rs` et
  `grep -rn 'starts_with("//")' crates --include=*.rs` sur **tout** `crates/` : `crates/kesh-db/src/backup.rs:1950-1951`
  (`corps_de_production`, même forme) ; `crates/kesh-db/src/repositories/closing_checks.rs:922` (`hors_tests`, un
  `split("#[cfg(test)]")` sur la source brute — tronqué aussi par un `"#[cfg(test)]"` écrit dans une chaîne) et `:928`
  (`decommente`, filtre `//` ligne à ligne) ; `crates/kesh-db/tests/test_schema_guard.rs:818-827` (profondeur de bloc
  comptée par `matches("/*")` et `matches("*/")` ligne à ligne, filtre `//` à la l. 825 — sans connaissance des
  chaînes). **Complété à la validation P3** (R3) : `crates/kesh-api/tests/admin_pat_denied_e2e.rs:124`
  (`strip_line_comments`, `l.split("//").next()` sur la source de `lib.rs`, mais aussi de `routes/admin.rs` (l. 573),
  `routes/fiscal_years.rs` (l. 581) et de tous les `.rs` de `src/` balayés à la l. 491 *(précisé à la validation P4,
  P4-1 ; la prémisse « pas de `//` dans une chaîne » n'est vérifiée que pour `lib.rs` — `main.rs:387-388` porte
  `http://` dans une chaîne, sans faux vert constaté)* — coupe aussi un `//` écrit dans une chaîne,
  limite que son doc-comment, l. 115-123, assume en s'appuyant sur le test compagnon
  `lib_rs_has_no_double_slash_inside_literals`). **Un analyseur juste existe déjà** : `crates/kesh-db/tests/lexique/mod.rs`
  (`decouper`, `blocs_de_test`, Story 28-4a, #642), employé par `letterings_lexical.rs` et
  `bank_account_closed_inventory.rs` — c'est la cible naturelle de l'extraction.
  **Depuis la validation P3** (C-32-1-38), la story n'en corrige **aucune** : celle d'`auto_backup.rs` (constat E-9 (3))
  rejoint les autres — **hors périmètre, non corrigées ici**, suivies par **[#727]** (ouverte le 2026-10-24 par
  l'orchestrateur, `technical-debt`, `P3`, sur la demande de la remédiation P1), dont le texte ne nomme ni les sites de
  `kesh-db`, ni celui d'`admin_pat_denied_e2e.rs`, ni `lexique`, ni le renvoi de E-9 (3) : commentaire à verser
  (*Commentaires à verser*, ci-dessous).

### Commentaires à verser (par l'orchestrateur ; la remédiation P2 n'ouvre aucune issue)

**Sur [#725]** (C-32-1-30) :

> Complément (Story 32-1, validation P2) : pendant une tentative, le panneau *Sauvegarde automatique* accole à la date de
> la tentative **en cours** le statut de la **précédente** — `record_attempt` ne pose que `last_attempt_at`
> (`crates/kesh-db/src/repositories/backup_schedule.rs:250-258`), `last_status` reste celui d'avant, et
> `AutoBackupSection.svelte:339-343` affiche « Dernière tentative : <maintenant> — réussie ». `record_failure` ne pose
> aucune date : l'état servi ne distingue pas « en cours » de « terminée en échec ». La Story 32-1b relit l'état après une
> réponse manquante (proxy 502/504 pendant l'écriture) et le dit au message (« la sauvegarde … s'écrit encore ») sans
> pouvoir le montrer ; le signal « tentative en cours » de la partie 1 le fermerait (par exemple un champ servi tiré de
> l'état du verrou, ou une date de fin de tentative).

**Sur [#727]** (C-32-1-35, **remplacé par C-32-1-38** : le texte à verser est celui de la validation P3, ci-dessous ;
celui de la validation P2 est gardé pour la trace — sa dernière phrase est fausse depuis le retrait de l'AC-a11 —, et
n'est **pas** à verser s'il ne l'a pas déjà été ; s'il l'a été, le texte P3 le corrige) :

> **Texte à verser (validation P3, 2026-10-24) :**
>
> Complément (Story 32-1, validations P2 et P3) — il remplace et corrige tout complément antérieur de la Story 32-1.
>
> **1. Le constat E-9 (3) de #711 rejoint cette issue.** Les trois helpers des gardes lexicales du `mod tests` de
> `crates/kesh-api/src/auto_backup.rs` lisent la source ligne à ligne : `code()` (l. 1121) n'écarte que les lignes
> entières qui commencent par `//` et tronque à la première ligne `#[cfg(test)]` — un commentaire de fin de ligne, un
> bloc `/* … */` et le contenu des chaînes y restent du code ; `argument_de` (l. 1159) compte les parenthèses écrites
> dans une chaîne ou un littéral de caractère ; `corps()` (l. 1182) coupe au premier `\n}`, chaîne comprise. Le sens
> dangereux est muet : un jeton **exigé** par une garde, écrit seulement dans un commentaire de bloc ou une chaîne, la
> satisferait. La Story 32-1a devait les corriger ; elle ne le fait plus — trois rédactions successives de la
> correction, trois défauts, chacun né du correctif précédent — et écrit seulement leur limite à leurs doc-comments, avec
> un renvoi ici. #711 se ferme en renvoyant E-9 (3) à cette issue, comme E-1 et E-2 à #725.
>
> **2. Un piège pour l'extraction, rencontré par la dernière rédaction.** Déclarer le module partagé par
> `#[cfg(test)] #[path = "../../kesh-db/tests/lexique/mod.rs"] mod lexique;` — l'attribut `#[cfg(test)]` **seul sur sa
> ligne**, comme `rustfmt` l'écrit, puisque `code()` ne tronque que sur une ligne dont le contenu est exactement
> `#[cfg(test)]` (`auto_backup.rs:1124`) — **en tête** d'un fichier de production
> dont une garde relit la source par `include_str!` et la tronque « au premier `#[cfg(test)]` » coupe tout le code de
> production : dans `auto_backup.rs`, l'assertion `try_lock_owned()` de `gardes_possedes` rougissait, et sa boucle des
> jetons interdits passait à vide sur ce fichier. La déclaration doit suivre le dernier item de production (juste avant
> `#[cfg(test)] mod tests`), ou la troncature se faire au `#[cfg(test)]` de `mod tests` (`blocs_de_test` de `lexique`),
> avec une assertion de montage sur un jeton de production connu. Le chemin `#[path]` se résout depuis `src/` hors
> module en ligne ; dans le `mod tests` en ligne, il se résoudrait sous `src/auto_backup/tests/`.
>
> **3. L'inventaire, sur tout `crates/`** (celui de l'issue ne couvrait que `kesh-api`) :
> `crates/kesh-db/src/backup.rs:1950-1951` (`corps_de_production`, même forme) ;
> `crates/kesh-db/src/repositories/closing_checks.rs:922` (`hors_tests`, `split("#[cfg(test)]")` sur la source brute)
> et `:928` (`decommente`) ; `crates/kesh-db/tests/test_schema_guard.rs:818-827` (blocs comptés ligne à ligne, sans
> les chaînes) ; `crates/kesh-api/tests/admin_pat_denied_e2e.rs:124` (`strip_line_comments`, `split("//")` sur la
> source de `lib.rs` : limite assumée par son doc-comment et par le test `lib_rs_has_no_double_slash_inside_literals`) ;
> et `auto_backup.rs` (point 1).
>
> **4. Un analyseur lexical juste existe déjà** : `crates/kesh-db/tests/lexique/mod.rs` (`decouper` : chaînes
> ordinaires et brutes, commentaires, caractères distingués des durées de vie, copie masquée de même longueur, fins de
> ligne gardées ; ses blocs `/* … */` ne s'imbriquent pas, alors que Rust les imbrique), inclus par `#[path]` dans
> `crates/kesh-api/tests/bank_account_closed_inventory.rs:61` et employé par `crates/kesh-db/tests/letterings_lexical.rs`.
> L'extraction demandée ici est donc un réemploi, non un balayeur neuf.

*Texte de la validation P2 (trace, à ne pas verser) :*

> Complément (Story 32-1, validation P2) : l'inventaire de cette issue ne couvrait que `kesh-api`. Sur tout `crates/` :
> `crates/kesh-db/src/backup.rs:1950-1951` (`corps_de_production`, même forme) ; `crates/kesh-db/src/repositories/closing_checks.rs:922`
> (`hors_tests`, `split("#[cfg(test)]")` sur la source brute) et `:928` (`decommente`) ;
> `crates/kesh-db/tests/test_schema_guard.rs:818-827` (blocs comptés ligne à ligne, sans les chaînes). Et **un analyseur
> lexical juste existe déjà** : `crates/kesh-db/tests/lexique/mod.rs` (`decouper` : chaînes ordinaires et brutes,
> commentaires, caractères distingués des durées de vie, copie masquée de même longueur), inclus par `#[path]` dans
> `crates/kesh-api/tests/bank_account_closed_inventory.rs`. La Story 32-1a le réemploie pour `auto_backup.rs` (et
> l'étend aux commentaires de bloc imbriqués) : l'extraction demandée ici est donc un réemploi, non un balayeur neuf.

### Fichiers touchés (prévus)

- **32-1a** : `crates/kesh-api/src/routes/invoice_pdf_service.rs` (commentaire, `mod tests`) ;
  `crates/kesh-api/src/auto_backup.rs` (doc-comments du `mod tests` seulement, AC-a10 ; ni module `lexique`, ni code des
  helpers — C-32-1-38) ; `crates/kesh-api/tests/audit_log_e2e.rs` ;
  `crates/kesh-api/tests/textes_coherents.rs` ; `crates/kesh-api/tests/admin_auto_backup_e2e.rs` ;
  `crates/kesh-db/src/repositories/company_invoice_settings.rs` (doc-comment) ;
  `crates/kesh-db/tests/company_invoice_settings_repository.rs` ;
  `frontend/src/routes/(app)/settings/fiscal-years/fiscal-years-page.test.ts`.
- **32-1b** : `frontend/src/lib/features/admin-backup/{AutoBackupSection.svelte, AutoBackupSection.test.ts,
  BackupScheduleBanner.svelte, BackupScheduleBanner.test.ts, backup-schedule.svelte.ts, backup-schedule.api.ts,
  backup-schedule.api.test.ts}` ; `frontend/src/lib/features/admin-restore/AdminRestorePanel.svelte` et un module
  partagé neuf sous `frontend/src/lib/shared/utils/` avec son test (prédicat d'issue inconnue extrait, AC-b2) ;
  `frontend/src/lib/shared/i18n-keys.test.ts` (`ATTENDU.sitesTotal`, AC-b8) ;
  `frontend/tests/e2e/backup-schedule.spec.ts` ; `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl` ;
  `docs/manual/fr/admin-manual.{tex,pdf}`. Aucun fichier Rust (C-32-1-37).
- **32-1d** : `crates/kesh-api/src/auto_backup.rs` ; `crates/kesh-api/src/routes/backup_schedule.rs` ;
  `crates/kesh-api/tests/{i18n_e2e.rs, admin_auto_backup_e2e.rs, textes_coherents.rs}` ;
  `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`.
- **32-1c** : `docs/manual/fr/admin-manual.{tex,pdf}` ; `docs/manual/fr/user-manual.{tex,pdf}` ;
  `docs/manual/fr/marketing-brochure.pdf` (régénéré par la dépendance au `.sty`, texte inchangé hors ses 16 signes,
  C-32-1-26, C-32-1-31) ;
  `docs/manual/shared/kesh-style.sty` ; `_bmad-output/implementation-artifacts/31-4-repetition-a-blanc.md`.
- **Toutes** : cette fiche, `sprint-status.yaml`, le registre `-321`.
- **CHANGELOG** (C-32-1-26) : la 0.15.0 est publiée (`## [0.15.0] — 2026-10-24`, `CHANGELOG.md:11`) et aucun bloc non
  publié n'existe au 2026-10-24. Convention du dépôt (celle que `scripts/prepare-release.sh` date, l. 237-255) : un
  bloc `## [0.16.0] — Non publié` en tête, sous la ligne `---`. La **première des clés à effet visible** qui fusionne —
  en pratique la première des **32-1c et 32-1d**, la 32-1b fusionnant après les deux (C-32-1-37) — le **crée** s'il
  n'existe pas encore sur `main` (une autre story de l'Epic 32 a pu le faire), les suivantes y ajoutent ;
  rubrique **`### Corrigé`** (le CHANGELOG est en français : `Ajouté`, `Modifié`, `Corrigé`, `Interne, invisible de
  l'utilisateur`). Une ligne pour la 32-1d (le refus d'un réglage de sauvegarde hors domaine dans la langue de
  l'instance), une pour la 32-1b (écran de la sauvegarde automatique : saisie gardée, réponse manquante relue, bandeau
  stable, horloge du serveur reculée dite telle, limite en deux unités) et une pour la 32-1c (manuels et brochure :
  procédure de répétition, signes imprimés) ; le drapeau `lastSuccessInFuture` servi par la 32-1d sans lecteur jusqu'à
  la 32-1b n'a pas de ligne propre (la 32-1b le dit) ; la 32-1a n'a aucun effet visible — pas d'entrée, et elle ne
  crée pas le bloc.

### Risques

- **32-1a** : faible ; le risque est le **test qui passe à vide** (`memory/tests-qui-prouvent-moins`) — d'où un rouge
  constaté avant chaque correctif et une mutation tuée par AC.
- **32-1b** : moyen ; trois changements de comportement d'écran, dont un état partagé (bandeau) ; l'E2E et les captures
  sont exigés ; elle dépend de la 32-1d (drapeau servi) — un développement fini avant la fusion de la 32-1d se rejoue
  après (AC-b10).
- **32-1d** : faible ; un champ ajouté à une réponse et un message traduit, sans lecteur frontend jusqu'à la 32-1b.
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

*Une sous-section par clé, chacune écrite par sa seule branche (Dev Notes, § *Pourquoi quatre clés*).*

### 32-1a

#### Agent Model Used

#### Completion Notes List

#### File List

### 32-1b

#### Agent Model Used

#### Completion Notes List

#### File List

### 32-1c

#### Agent Model Used

#### Completion Notes List

#### File List

### 32-1d

#### Agent Model Used

#### Completion Notes List

#### File List

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
- 2026-10-24 — **Validation P2** (Opus 5.5, deux lentilles en contexte frais, lecture seule ; prompt
  `321-validate-prompt-p2.md` ; rapports `kesh-gate-logs/321-validate-p2-R.md` et `321-validate-p2-F.md`). Lentille R :
  **0 CRITICAL, 0 HIGH, 3 MEDIUM, 5 LOW** (R1 à R8) ; lentille F : **0 CRITICAL, 0 HIGH, 4 MEDIUM, 8 LOW** (F1 à F12).
  Recoupements : F3 = R2 ; F10 = R5 (a) + R6 (b) ; F9 (b) = R1 — soit **6 MEDIUM distincts** (R1, R2 = F3, R3, F1, F2,
  F4) et **12 LOW distincts** (R4, R5, R6, R7, R8, F5, F6, F7, F8, F9 (a), F11, F12). Remédiation (Opus 5.5), chaque
  affirmation relue au code :
  - **F1** (MEDIUM ; origine : rédaction d'origine, C-32-1-8, étendue par la remédiation P1 à la relecture de l'AC-b2) —
    `applyState` posait `schedule` sous une saisie gardée, et « Enregistrer » partait avec la `version` de la réponse :
    le réglage d'un autre administrateur écrasé sans 409, la Dev Note disant l'inverse. AC-b1 : `formBase`, `save` envoie
    `formBase.version`, le 409 existant joue ; Vitest (iii) « concurrence », **M-b9** ; Dev Note corrigée. C-32-1-29.
  - **F2** (MEDIUM ; origine : rédaction d'origine, C-32-1-9, formulée par la remédiation P1) — la relecture tombe pendant
    l'écriture (proxy 502/504, tâche `tokio::spawn` qui continue) et montre la date de la tentative en cours avec le
    statut de la précédente ; rien dans l'état servi ne distingue « en cours » (`record_failure` ne pose aucune date).
    Formulation honnête : message neuf (« … ou s'écrit encore … rechargez la page dans quelques minutes ») et phrase du
    manuel sans « où se lit le résultat » ; l'affichage, défaut préexistant, en commentaire de **#725** (sa partie 1
    prévoit le signal). C-32-1-30.
  - **F3 = R2** (MEDIUM ; origine : rédaction d'origine, constat élargi en P1 sans aller jusqu'au symptôme) — classe
    recensée par **U+FFFD** : 26 / 9 / 16 dans les trois PDF ; 17 `⚠️` + 9 signes de listings (`✓`, `✗`, `═`, l. 1547,
    3313, 3318, 3320) ; 15 `\faCheckCircle` + 1 `\faExchangeAlt` (brochure). Mécanique, même fichier : intégrée à la
    32-1c (deux replis d'icônes, `literate` du style `kesh`) ; AC-c7 contrôle **U+FFFD → 0** dans les trois PDF ;
    AC-c9 : brochure identique hors ses 16 signes. Aucune issue. C-32-1-31.
  - **F4** (MEDIUM ; origine : remédiation P1) — #727 ouverte depuis : bilan **−3 + 3 = 0** issue, **−1** bug (#727
    `technical-debt`, sans `bug`) ; *Origine* et *Inventaire* corrigés, lien `[#727]`. L'entrée « Validation P1 »
    ci-dessus (« −3 + 2 ») était juste à sa date — #727 a été ouverte après son commit — et n'est pas réécrite. C-32-1-35.
  - **R1** (MEDIUM ; origine : remédiation P1 — **recyclage** sur l'AC-a11) — M-a7 indécidable (un retrait non gourmand ne
    trouve aucun `*/` dans (3) ni dans `lib.rs`). M-a7 en deux formes : **M-a7a** (le balayeur privé de l'état chaîne),
    **M-a7b** (retrait non gourmand) ; assertion **(3')**, fermée par `/* c */` après l'appel, qui tue les deux.
    C-32-1-32.
  - **R3** (MEDIUM ; origine : rédaction d'origine) — à la première sauvegarde, `derniere = 0` : le seuil vaut 1,2 ×
    documents + 256 Mio, la base n'est couverte que par 0,2 × documents + 256 Mio, quelle que soit sa taille. Phrase
    juste prescrite (AC-b7). C-32-1-33.
  - **Constat neuf de la remédiation** (en complétant R4) : un analyseur lexical juste **existe déjà**
    (`crates/kesh-db/tests/lexique/mod.rs`, `decouper`, Story 28-4a) ; l'AC-a11 en prescrivait un second. AC-a11
    réécrite : `decouper` réemployé par `#[path]`, étendu aux blocs imbriqués, helpers sur la copie masquée (fins de ligne
    gardées — ce qui règle F9 (a) — ; la limite « `\n}` dans une chaîne » de `corps()` disparaît), assertion (6) ;
    modules de la 32-1a inchangés en nombre (kesh-db y était). C-32-1-32.
  - LOW : **R4** — inventaire complété sur tout `crates/` (`kesh-db/src/backup.rs:1950-1951`, `closing_checks.rs:922`,
    `:928`, `tests/test_schema_guard.rs:818-827`) et `lexique` nommé ; commentaire à verser sur **#727**. **R5 = F10 (a)**
    — un rejet qui n'est pas une `ApiError` (200 au corps illisible : `request()` rend `undefined`) → issue inconnue,
    Vitest. **R6 = F10 (b)** — `loadError` remis à `null` au départ de `runNow`, Vitest, **M-b10**. C-32-1-34. **R7** — la
    phrase du manuel dit « 5xx, 503 compris ». **R8** — quatre commentaires nommés (getter l. 14, en-tête du test).
    **F5** — modules de la 32-1b : 4 au grain des crates et paquets, **7** au grain des modules métier ; signal déclaré
    ci-dessous, sans découpage. C-32-1-36. **F6** — CHANGELOG : « la première des 32-1b et 32-1c ». **F7** — brochure
    d'avant tirée de `git merge-base origin/main HEAD`, jamais de `HEAD:`. **F8** — assertion de montage « trois `NULL`
    après (t1) ». **F9 (a)** — fins de ligne : réglé par `decouper` et l'assertion (6) ; **(b)** = R1. **F11** — Dev Agent
    Record en trois sous-sections ; 32-1b : `make admin`, seul `admin-manual.pdf` commité. **F12** — constat P2 B2-L4
    nuancé (« muet » sur la sortie standard seulement).
  - **Tendance** : P1 **4 MEDIUM** distincts → P2 **6 MEDIUM** distincts. ⚠️ **Signal de la règle de splitting
    (sévérité égale, MEDIUM → MEDIUM), déclaré au Project Lead** : quatre des six sont des défauts **d'origine** non vus
    en P1 (F1, F2, F3 = R2, R3) — la revue trouve, elle ne tourne pas en rond (amendement D5) ; deux viennent de la
    remédiation P1 (F4, comptable ; R1, **recyclage** de l'AC-a11, qui reçoit en P2 sa deuxième réécriture) ; et la
    32-1b dépasse cinq modules au grain métier (F5). Le découpage n'est pas fait par la remédiation ; s'il est décidé,
    la coupe la moins coûteuse sépare la 32-1b en un écran (E-6, E-7, E-8, L-B6 : `admin-backup`, `admin-restore`,
    `shared/utils`, i18n, manuel) et un serveur (L-B5, L-B7 : `auto_backup`, `routes/backup_schedule`, bandeau, i18n,
    manuel), au prix d'un ordre de plus sur le PDF et les catalogues.
  - Registre : **C-32-1-29 à C-32-1-36** ; amendements datés de C-32-1-8, 16, 20, 26. Propagation grepée dans la fiche,
    le registre et le prompt P3 (`−3 + 2`, `applyState`, `schedule.version`, `où se lit le résultat`, `M-a7`, `M-b8`,
    `huit`, `grep -c "⚠"`, `texte inchangé`, `git show HEAD:`, `la clé qui fusionne la première`, `256 Mio`, `Un seul
    balayeur`). Aucune issue neuve ; deux commentaires à verser par l'orchestrateur (#725, #727, *Commentaires à
    verser*). **Six MEDIUM remédiés : une passe P3 complète est due** (`321-validate-prompt-p3.md`, Sonnet).
- 2026-10-24 — **Découpage de la 32-1b** (Opus 5.5, avant la validation P3 ; décision de l'orchestrateur, règle de
  splitting préventif de `CLAUDE.md`, amendement D5 ; C-32-1-37). Motifs : la 32-1b touchait **7 modules** au grain
  métier (C-32-1-36), et la P2 a levé le signal de sévérité **MEDIUM → MEDIUM avec un recyclage** (R1, deuxième
  réécriture de l'AC-a11) — ce que l'amendement D5 compte pour découper. Signal déclaré au Project Lead (registre,
  C-32-1-37). Coupe, celle que proposait la remédiation P2, ajustée au code :
  - **32-1b « écran »** (E-6, E-7, E-8, L-B6 ; L-B3 et L-B5 pour le manuel et le bandeau) : AC-b1, AC-b2, AC-b3, AC-b4
    (réduit au bandeau, au type `BackupSchedule` et aux fixtures), AC-b5, AC-b7 (manuel entier, L-B3 compris : une phrase
    sur un comportement existant, sans code), AC-b8 (deux clés neuves, une modifiée, **+2**), AC-b9 (a)–(d), AC-b10,
    AC-b11 (neuf mutations : M-b1 à M-b4, M-b7 à M-b10, et **M-b11** neuve — le bandeau qui ignore le drapeau, la M-b5
    d'origine étant côté serveur) ; **5 modules** au grain métier, 3 au grain des paquets ; **aucun fichier Rust**.
  - **32-1d « serveur »** (L-B5 drapeau servi, L-B7) : AC-d1 (ex-AC-b4, moitié serveur), AC-d2 (ex-AC-b6), AC-d3 (une
    clé, **+1**), AC-d4 (ex-AC-b9 (e)), AC-d5 (gates, E2E complet : la réponse servie change), AC-d6 (M-d1 = ex-M-b5,
    M-d2 = ex-M-b6) ; **3 modules** au grain métier, 2 au grain des crates ; aucun fichier du frontend ni du manuel.
  - Aucune AC ne tombe entre les deux : chaque ligne de l'AC-b4 d'origine est à l'une ou à l'autre ; L-B3 n'a pas de
    code ; le manuel (AC-b7) ne décrit que l'écran et un seuil existant. Numéros de la 32-1b gardés, renvois laissés
    (AC-b6, T-b1, M-b5, M-b6).
  - **Ordre de fusion** : 32-1a quand elle est prête ; 32-1c et 32-1d dans l'ordre où elles sont prêtes ; **32-1b après
    la 32-1c et la 32-1d**. La 32-1d hérite des trois fichiers de code partagés avec la 32-1a (rebase et gate complet de
    la seconde) ; la 32-1b et la 32-1a ne partagent plus rien. Mots-clés : `closes #711` sur la dernière fusionnée des
    32-1a, 32-1b, 32-1d — la 32-1d, toujours avant la 32-1b, porte `refs #711`. CHANGELOG : bloc créé par la première
    fusionnée des 32-1c et 32-1d, une ligne pour la 32-1d. Bilan d'issues inchangé (**0** issue, **−1** bug) : le
    découpage n'ouvre ni ne ferme rien.
  - `sprint-status.yaml` : clé `32-1d-defauts-sauvegarde-serveur` (`ready-for-dev`) ; prompt P3 amendé (le découpage
    vérifié, partie D lue). Registre : C-32-1-37 ; amendements datés de C-32-1-17, 18, 27, 28, 36. Propagation grepée
    dans la fiche (`trois clés`, `M-b5`, `M-b6`, `AC-b6`, `T-b1`, `is_success_in_future`, `i18n_e2e`, `+3`,
    `32-1b et 32-1c`, `32-1a et 32-1b`).
  - *Annotation de la validation P3 (2026-10-24, F2 = R4, F3) : le motif écrit ci-dessus était faux à sa date sur un
    point — le recyclage (R1 de la P2) porte sur l'AC-a11, à la **32-1a**, que ce découpage ne touche pas ; seule la
    dispersion (7 modules au grain métier) justifiait de découper la 32-1b, et elle suffit. Et l'amendement de C-32-1-26
    (« première des 32-1b et 32-1c ») n'avait pas été propagé : il l'est par la P3. Voir C-32-1-37 (amendé) et
    C-32-1-38.*
- 2026-10-24 — **Validation P3** (Sonnet 5.5, deux lentilles en contexte frais, lecture seule ; prompt
  `321-validate-prompt-p3.md` ; rapports `kesh-gate-logs/321-validate-p3-R.md` et `321-validate-p3-F.md`). Lentille R :
  **0 CRITICAL, 0 HIGH, 1 MEDIUM, 4 LOW** (R1 à R5) ; lentille F : **0 CRITICAL, 0 HIGH, 2 MEDIUM, 8 LOW** (F1 à F10).
  Recoupements : **F1 = R1** ; **F2 = R4** (F2 classé MEDIUM, R4 LOW : retenu à la plus haute) — soit **2 MEDIUM
  distincts** (R1 = F1, F2 = R4) et **11 LOW distincts** (R2, R3, R5, F3 à F10). La lentille F déclare une entorse (une
  redirection vers `/tmp/claude-1000/`, fichier non relu, hors dépôt). Remédiation (Opus 5.5) :
  - **R1 = F1** (MEDIUM ; origine : remédiation P2, troisième rédaction de l'AC-a11) — l'inclusion de `lexique` « en
    tête » d'`auto_backup.rs` tronquait `code()` avant tout le code de production (`gardes_possedes` rouge sur
    `try_lock_owned()`, muette sur ses jetons interdits). **Décision de l'orchestrateur : retrait**, sur le signal de
    recyclage (trois rédactions, trois défauts, chaque remédiation introduisant le suivant). L'AC-a11, M-a7a, M-a7b et
    le réemploi de `lexique` sortent de la 32-1a ; E-9 (3) est renvoyé à **[#727]** (commentaire P3 à verser, qui
    remplace celui de la P2 et consigne le piège) ; l'angle mort est écrit aux doc-comments des helpers (AC-a10 élargi,
    commentaires seuls). #711 se ferme toujours par la dernière des 32-1a et 32-1b, son corps de PR écrivant « E-9 (3)
    suivi par #727 » ; bilan d'issues inchangé (**0** issue, **−1** bug). Mutations de la 32-1a : **huit** (M-a1a à
    M-a6). C-32-1-38 ; annotations datées de C-32-1-6, 20, 32.
  - **F2 = R4** (MEDIUM ; origine : découpage) — motif du découpage corrigé (dispersion seule pour la 32-1b ; le
    recyclage était sur la 32-1a) : amendement daté de C-32-1-37, annotation de l'entrée « Découpage » ci-dessus. Grain
    métier de la 32-1a écrit après le retrait : **6** (3 au grain des crates) — au-delà de cinq, **signal déclaré au
    Project Lead**, sans découpage (AC indépendants, recyclage sorti). C-32-1-38.
  - LOW : **R2** — `i18n-keys.test.ts` (`ATTENDU.sitesTotal` 2332, l. 715) nommé à l'AC-b8 et aux *Fichiers touchés*
    de la 32-1b, relevé de 2 avec commentaire daté. **R3** — `admin_pat_denied_e2e.rs:124` ajouté à l'*Inventaire* et au
    commentaire de #727. **R5** — phrase de l'espace disque : la base n'est comptée à travers la dernière sauvegarde que
    si celle-ci dépasse les documents (`seuil_d_espace` prend le `max`) ; « tant que la dernière sauvegarde réussie n'est
    pas plus grande que les documents, en particulier avant la première ». **F3** — amendement daté de C-32-1-26
    (« première des 32-1c et 32-1d »). **F4** — `CHANGELOG.md` nommé parmi les fichiers communs des 32-1b, 32-1c,
    32-1d (un seul bloc, conflit prévu au rebase de la seconde). **F5** — AC-c9 : commande exacte, `sed $'…'` sur les
    caractères entiers, `cmp` sur deux substitutions de processus. **F6** — en-tête de la clé 32-1b de
    `sprint-status.yaml` annoté (L-B7 à la 32-1d). **F7** — AC-d5 : `cargo nextest run -p kesh-i18n` au gate ciblé.
    **F8** — AC-b2 : le module du prédicat importe `isApiError` de `./api-error`. **F9** — AC-b7 : G22 nommée, aucune
    tournure « plus de deux intervalles ». **F10** — AC-b2 : pour le 200 au corps illisible, la tentative est terminée
    (réussie ou en échec) ; la limite du message unique est écrite (vrai pour TIMEOUT, NETWORK_ERROR, 502, 504).
  - **Tendance** : P1 4 MEDIUM → P2 6 → P3 **2** distincts, dont un recyclage (R1, retiré) et un motif de compte rendu
    (F2). La remédiation P3 touche une AC de code (l'AC-a11 retirée, l'AC-a10 élargie à des commentaires) : **une passe
    P4 est due** — ciblée, une lentille (regression hunter sur la spec), Haiku, sur le seul diff de cette remédiation
    (`321-validate-prompt-p4.md`) ; la boucle ne se clôt que sur une passe sans finding au-dessus de LOW.
  - Propagation grepée dans la fiche, le registre et `sprint-status.yaml` (`AC-a11`, `lexique`, `M-a7`, `en tête`,
    `decouper`, `E-9 de #711`, `neuf ;`, `est écrite`, `Tant qu'aucune sauvegarde`, `32-1b et 32-1c`, `2332`). Les
    entrées passées du Change Log (P1, P2) ne sont pas réécrites.
- 2026-10-24 — **Validation P4** (Haiku, passe ciblée, une lentille *regression hunter* sur `8d4c29df..7e4b7b5f` ;
  rapport hors dépôt `321-validate-p4.md`) : **0/0/0/4** — **validation terminée**. Tendance des MEDIUM distincts :
  P1 4 → P2 6 → P3 2 → P4 0. LOW appliqués par l'orchestrateur : P4-1 (`strip_line_comments` sert aussi
  `routes/admin.rs`, `routes/fiscal_years.rs` et le balayage de `src/` — précisé à l'inventaire, suivi sur #727) ;
  P4-2 (l'import d'`AdminRestorePanel.svelte:20` n'est pas le modèle) ; P4-3 (l'attribut `#[cfg(test)]` seul sur sa
  ligne — précisé au commentaire de #727) ; P4-4 (annotation P3 de la clé 32-1b déplacée en fin de ligne, datée).
  Signal déclaré au Project Lead, sans découpage : la 32-1a touche six modules au grain métier (C-32-1-38).

[#711]: https://github.com/guycorbaz/kesh/issues/711
[#714]: https://github.com/guycorbaz/kesh/issues/714
[#716]: https://github.com/guycorbaz/kesh/issues/716
[#710]: https://github.com/guycorbaz/kesh/issues/710
[#725]: https://github.com/guycorbaz/kesh/issues/725
[#726]: https://github.com/guycorbaz/kesh/issues/726
[#727]: https://github.com/guycorbaz/kesh/issues/727

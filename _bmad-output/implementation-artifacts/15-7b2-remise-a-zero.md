# Story 15.7b2 : La remise à zéro laisse sa trace, vide tout, et garde la société

Status: done

<!-- Née le 2026-10-08 du découpage de la 15-7b (choix C-15-7-31), à la passe de validation P4 :
     coupe Volet A / Volet B que la section « Dérogation règle de splitting » de la 15-7b prévoyait
     « sans nouvel arbitrage » si un défaut né d'une remédiation revenait à sévérité égale (R4-2, F-1 de
     la P4). Cette fiche porte le Volet B (la remise à zéro, #528, #279) et les invariants qui s'y
     rapportent ; la 15-7b1 porte le Volet A (la démonstration).
     Choix applicables de `epic-15-choix-autonomes.md` : C-15-7-5 (la remise à zéro inscrit son propre
     geste), C-15-7-9 (#528 : identité de la société préservée), C-15-7-10 (#279 : vidage dérivé de la
     liste canonique), C-15-7-11 (les trois gardes sous le verrou de la transaction qui efface),
     C-15-7-13 (atomicité prouvée par un déclencheur de test), C-15-7-16 (`onboarding_state` remis à
     zéro en place), C-15-7-22 (ordre des verrous, exception au Pattern 5, rejeu sur interblocage),
     C-15-7-23 (#528 sans société : helper de rattachement unique), C-15-7-24 (partition gardée sur le
     schéma — amendé par C-15-7-28), C-15-7-25 (contrôles FK rétablis avant le commit ; même release que
     la 15-7a2 et la 15-7b1 ; sa branche `detach()` est remplacée par C-15-7-32), C-15-7-27 (prédicat de
     rejeu gardé par le type), C-15-7-28 (chaîne de clés sans traverser une table préservée, valeurs du
     stub), C-15-7-31 (découpage), C-15-7-32 (connexion fermée à la libération, règle 3, colonnes par
     le schéma). Version d'avant découpage : `9bbc52de` (fiche 15-7b, P3 appliquée).
     Validation : P1 (sur la 15-7), P2 à P4 (sur la 15-7b) appliquées ; P1 de cette fiche appliquée
     (1 HIGH, 4 MEDIUM — #542 fermée ici, C-15-7-34, C-15-7-36) ; P2 appliquée (4 MEDIUM — réparation
     de #528 au démarrage, C-15-7-38 ; clause de coupe précisée, C-15-7-39) ; P3 appliquée (2 MEDIUM nés
     de la remédiation P2, 13 LOW) : **coupe déclenchée et faite selon la cause** — la réparation des
     installations déjà atteintes (AC 14, test 15) sort en **15-7b3**, cette fiche garde la prévention
     (C-15-7-40 à C-15-7-42) ; P4 appliquée (3 MEDIUM, 7 LOW : une règle et une fonction pour les
     principaux orphelins — clés révoquées puis repointées, C-15-7-45 ; réparation de #542 passée à la
     15-7b3, C-15-7-46 ; sortie de la démonstration dite au manuel, C-15-7-48) ; P5 appliquée (1 MEDIUM, 8 LOW : la recette
     de sortie de la démonstration dépend de la 15-11a, qui transmet `KESH_PRODUCTION_RESET` au conteneur,
     #550 — C-15-7-51 ; tableaux du manuel, C-15-7-52) : **validation close**. -->

**Issues** : `closes #434`, `closes #279` — **sur la PR**, pas sur les
commits intermédiaires (`refs #N`) : le dépôt merge en squash. Dernière des sous-stories de la 15-7, c'est
elle qui ferme #434. **`refs #528`** (C-15-7-40) : cette fiche en ferme la **cause** (la remise à zéro
et `language` ne laissent plus personne rattaché à une société effacée) ; la **15-7b3** répare les
installations déjà atteintes et porte `closes #528`. **`refs #542`** (C-15-7-46) : cette fiche en ferme
la **cause** (le démarrage n'ajoute plus de société provisoire) ; la 15-7b3 supprime au démarrage les
sociétés provisoires déjà ajoutées et porte `closes #542`. **`refs #534`** (libellé du refus de la remise
à zéro, Dev Notes). **`refs #540`** et **`refs #538`**, jamais `closes` (R8 de la P1) : #540 vise aussi
`restore_tables_in_tx`, que la story ne touche pas — la part `reset_demo` livrée se signale en
commentaire sur #540 (orchestrateur) ; #538 reste ouverte. **`refs #550`** (C-15-7-51) : les compose
de production ne transmettent pas `KESH_PRODUCTION_RESET` au conteneur ; la **15-11a** le corrige
(`closes #550`), cette fiche n'en dépend que pour la recette de sortie de la démonstration.

**Dépendances** : **la 15-7b1, mergée** (et donc la 15-7a2 et la 15-7a1) — elles posent
`onboarding::record_step_completed_in_tx`, `onboarding::lock_state_in_tx`, `SeedError` et son acteur,
l'entrée `company.created` du chemin « aucune société » de `language`, le helper `audit_sequence`, le
fichier `crates/kesh-api/tests/onboarding_audit_e2e.rs`, et l'ordre des verrous des routes de
production. **La 15-7b3 dépend de celle-ci** (C-15-7-40) — en particulier de
`companies::reattach_orphan_principals_in_tx` (AC 4), la règle unique des principaux orphelins qu'elle
appelle au démarrage et à la restauration (C-15-7-45). ⛔ **La 15-11a (`15-11a-compose-transmet-la-configuration`,
#550) est mergée AVANT celle-ci** (C-15-7-51, F5-1 de la P5) : `docker-compose.yml` et
`docker-compose.prod.yml` — le premier est celui que le manuel fait télécharger (`admin-manual.tex`,
« Étape 2 — Récupérer le `docker-compose.yml` ») — portent une liste `environment:` explicite, sans
`env_file`, et ne transmettent pas `KESH_PRODUCTION_RESET` ; poser la variable dans `.env` n'atteint donc
pas le processus, et la recette de sortie de la démonstration que cette fiche écrit au manuel (AC 11,
lignes `:691` et `:1314`) serait inexécutable sur l'installation documentée. Le développement peut
commencer avant ; **la PR ne se merge pas** tant que `grep -n "PRODUCTION_RESET" docker-compose.yml
docker-compose.prod.yml` ne rend pas une ligne dans chacun des deux fichiers (contrôle T8, sortie collée
au Dev Agent Record). Cette fiche **ne modifie pas** les compose. ⚠️ **Même release que la 15-7a2 et la 15-7b1** (C-15-7-25, C-15-7-31) : entre la 15-7a2
et celle-ci, une remise à zéro à l'étape ≤ 2 effacerait sans trace les entrées neuves de la 15-7a2.
⚠️ **Renvois de ligne** : ils valent sur la base `b74c3dac` (avant les 15-7a1, 15-7a2 et 15-7b1, qui
réécrivent `routes/onboarding.rs` et `kesh-seed`) ; à relocaliser **par symbole** au moment du
développement. **Il en va de même des numéros de ligne des manuels** (`admin-manual.tex:…`,
`user-manual.tex:…`, AC 11, T7 ; F5-2 de la P5) : `origin/main` a déjà modifié les deux `.tex` depuis
`b74c3dac`, et les 15-7a2 et 15-7b1 réécrivent les mêmes paragraphes ; ils se relocalisent **par le
texte cité** (grep de la phrase ou de la valeur, AC 11) après rebase. Le commentaire « rare hors bootstrap » d'`ensure_company_with_language` (`:866-869`)
est reformulé **ici** (AC 10) : la 15-7a2 trace la branche sans en changer le commentaire.
⚠️ **Conflits attendus au merge** : registre des routes, `ACTIONS`, quatre `.ftl`, `## [0.13.0]` du
CHANGELOG — recompter depuis la source.
⚠️ **Numérotation** : les AC et les tests gardent les numéros de la 15-7b (traçabilité des passes P1 à
P4) ; les numéros absents sont à la 15-7b1.

## Story

En tant que **fiduciaire ou réviseur qui lit le journal d'audit d'une installation**,
je veux qu'**une remise à zéro, qui efface la piste, y inscrive son propre geste**,
afin qu'**un effacement de la piste ne soit jamais muet**, et qu'une installation remise à zéro reparte
réellement de rien **sans que l'utilisateur connecté se retrouve rattaché à une société effacée**.

## Décompte de modules — signal D5 déclaré

À la granularité retenue par C-15-7-8 : `kesh-db/repositories/companies` (`reset_to_stub_in_tx`,
`insert_stub`, `reattach_orphan_principals_in_tx`, constantes du stub), `kesh-db/repositories/onboarding`
(`reset_state_in_tx`), `kesh-db/backup` (`RESET_PRESERVED_TABLES`, `reset_cleared_tables`), `kesh-seed`, `kesh-api/routes/onboarding`
(handler `reset`, branche « aucune société »), `kesh-api/auth/bootstrap` (constantes, `insert_stub`, garde de #542),
`kesh-api/audit_labels`, `kesh-i18n` — **huit modules de code**, plus un test de composant frontend.
Le seuil de cinq est franchi : **le signal est déclaré au Project Lead**. La coupe Volet A / Volet B ne
l'a pas fait repasser sous le seuil — le Volet A n'en portait que peu.

### Dérogation règle de splitting

**Justification** : les huit modules servent **une seule transaction** — la remise à zéro — et deux
défauts que l'inventaire § 6 établit inséparables : préserver la société (#528 : `companies`,
`bootstrap`, branche « aucune société ») **sans** vider toutes ses tables (#279 : `backup`) rattacherait
à la société conservée les données de la démonstration ; et l'entrée d'audit (`audit_labels`,
`kesh-i18n`) s'écrit **dans** cette transaction, après le rattachement. Un découpage par module
produirait des sous-stories dont aucune ne serait testable de bout en bout.
**Coupe prédéclarée** : si une passe suivante remonte un défaut **né d'une remédiation** (recyclage, au
sens de l'amendement D5) à sévérité égale ou supérieure **à la sévérité maximale de la passe
précédente** — non à celle du seul défaut remédié (C-15-7-39 ; lecture fixée à la P2, où des MEDIUM nés
de la remédiation suivaient une P1 à HIGH : clause non déclenchée) —, la fiche se coupe **sans nouvel arbitrage** en
une **story-zéro `kesh-db`** — `companies::{reset_to_stub_in_tx, insert_stub,
reattach_orphan_principals_in_tx}`, `onboarding::reset_state_in_tx`, `backup::{RESET_PRESERVED_TABLES, reset_cleared_tables}`, tests 5 (part
colonnes), 6f, 10 et 10b, **sans appelant** — puis une story qui porte `reset_demo`, les handlers, le
bootstrap et l'audit. ⚠️ Le constat de recyclage se **vérifie par versions de la fiche aux bornes des
passes** (`git show <commit>:<fiche>`), jamais de mémoire : c'est faute de l'avoir fait que la
dérogation de la 15-7b reposait sur un constat faux (R4-2 de la P4).
**Risque accepté** : une boucle de validation plus longue sur une fiche de huit modules.
**Clause déclenchée à la P3** (C-15-7-40) : F3-1 et F3-2, MEDIUM, sont nés de la remédiation P2 (vérifié
par versions : `repair_orphan_principals` et `principals_reattached` comptent **0** occurrence au commit
`327ea9df` de la fiche, **10** occurrences sur **9** lignes au commit `ffcdcf8d` — R4-4 de la P4), après une P2 à MEDIUM. **La coupe ne suit pas l'axe
prédéclaré** (story-zéro `kesh-db`) mais la **cause** : les deux MEDIUM portent sur la réparation des
installations déjà atteintes, et une story-zéro `kesh-db` ne les aurait pas fermés (même motif que
C-15-7-39). La réparation (ex-AC 14, test 15) sort en **15-7b3** ; cette fiche garde la prévention. Elle
reste à huit modules (`auth/bootstrap` y demeure pour #542) : la clause et son axe `kesh-db` valent pour
les passes suivantes, par rapport à la sévérité maximale de la P3 (MEDIUM).
**Clause à la P4 — déclarée, non appliquée** (C-15-7-45, C-15-7-48) : deux MEDIUM de la P4 suivent une
remédiation à sévérité égale (MEDIUM) — F4-1/R4-1, la règle C-15-7-41 appliquée à un chemin sur trois
(résidu de **propagation** d'une décision, non défaut de conception), et F4-2/R4-2, une ligne de manuel
écrite à la P3 (F3-7). Décision de l'orchestrateur : **pas de coupe** — l'axe prédéclaré (story-zéro
`kesh-db`) ne fermerait ni un défaut de propagation entre fiches sœurs ni une ligne de manuel (même motif
que C-15-7-39 et C-15-7-40) ; **signal D5 déclaré au Project Lead** (Change Log P4). La remédiation
**retire** de cette fiche la réparation de #542 (C-15-7-46) et y **pose** la règle des principaux
orphelins (C-15-7-45) : toujours **huit modules**.

## Ce que l'inventaire a établi, et qui n'est pas dans l'issue

1. **`reset` n'est pas atteignable par un jeton d'API** (`admin_routes`, `lib.rs:312`, derrière
   `require_not_pat` ; `tests/admin_pat_denied_e2e.rs:55`). L'acteur est celui que la 15-7b1 threade
   (`(user_id, api_key_id)`, `for_actor`) ; pour `reset`, `api_key_id` est toujours `None`, mais le
   même chemin sert.
2. **`reset_demo` n'a aucune transaction** (`kesh-seed/src/lib.rs:233-311`) : **huit** `DELETE` en
   `autocommit` sur une connexion `FOREIGN_KEY_CHECKS=0`, puis `delete_state`/`init_state` sur le pool.
   Et le handler relâche son verrou **avant** l'effacement (`routes/onboarding.rs:281-291`) — or c'est
   lui seul qui porte les **trois** gardes (`step >= 7` `:257`, `!is_demo && step > 2` `:266`,
   `KESH_PRODUCTION_RESET` `:276`). Une progression concurrente entre le commit et l'effacement efface
   une installation de production en cours sans le drapeau.
3. **#528 — après une remise à zéro, l'utilisateur désigne une société effacée.** `reset_demo` efface
   `companies` sous `FOREIGN_KEY_CHECKS=0`, la cascade `fk_users_company`
   (`20260419000002_users_company_id.sql:32`) ne joue pas, et rien ne repointe `users.company_id`
   (seul `UPDATE users SET company_id` du dépôt : la migration d'origine, `:13`). `set_language`
   recrée une société **d'un autre id** (branche « aucune société » de `ensure_company_with_language`,
   `routes/onboarding.rs:865-883`, qui ne rattache personne) ; le JWT, forgé depuis `user.company_id`
   (`routes/auth.rs:247`, `:437`, `:539`), désigne la morte, et `get_company_for`
   (`helpers.rs:61-72`) répond **500** aux routes scopées qui passent par lui (celles qui lisent
   `current_user.company_id` directement rendent des listes vides — F3-4 de la P3). ⚠️ **Une installation déjà atteinte sous
   v0.12.x est SANS société** jusqu'au `language` suivant : la réparation doit donc vivre **aussi** dans
   cette branche, et la remise à zéro doit accepter une base sans société. ⚠️ **Et l'état le plus
   probable n'est ni l'un ni l'autre** (F-1 de la P2) : le parcours démo → remise à zéro (v0.12.x) →
   `language` (nouvelle société, personne rattaché) → production → `finalize` **passe**, aucune route
   d'onboarding ne lisant la société de l'utilisateur (`get_company`, `routes/onboarding.rs:916`,
   `MIN(id)`) ; l'installation arrive finalisée, **une** société, `users.company_id` mort — et ni la
   remise à zéro (refusée à l'étape ≥ 7), ni `language` (plus appelé) ne la réparent. Sa réparation —
   **au démarrage** et à la **restauration** d'une sauvegarde — est portée par la **15-7b3**
   (C-15-7-40) ; cette fiche ferme la cause. ⚠️ **Les clés d'API aussi** désignent la société effacée
   (`reset_demo` v0.12.x ne vide pas `api_keys`, `kesh-seed/src/lib.rs:251-299`) : inertes aujourd'hui
   (`get_company_for` répond 500), elles seraient **réveillées** par un repointage nu — alors qu'elles ont
   pu naître en démonstration (C-15-7-41) ; révoquées sans être repointées, elles resteraient des clés
   étrangères pendantes, invisibles sur la page des clés (`list_by_company` filtre par société). D'où une
   règle unique, posée ici et appelée par la 15-7b3 : **révoquer, puis repointer** (AC 4, C-15-7-45).
4. **#279 — la remise à zéro n'est pas exhaustive.** Elle laisse `invoices`, `credit_notes`,
   `contacts`, `products` et une vingtaine d'autres tables d'une société effacée. La liste canonique
   des tables applicatives existe et est gardée par un test fail-loud :
   `kesh_db::backup::TABLES_TO_TRUNCATE` (`backup.rs:34-86`, 39 tables, ordre enfants → parents,
   `backup_inventory_matches_schema`).
5. **Les fichiers de documents ne sont pas des tables** : `invoices.pdf_storage_path` (PDF figés,
   #387) et `imported_supplier_invoices.storage_path` désignent des fichiers de `KESH_DOCUMENTS_DIR`
   (`config.rs:946-949`). La remise à zéro efface les lignes, pas les fichiers (L-2 de la P4).
6. **#528 et #279 sont liés plus étroitement que l'issue ne le dit** : préserver l'identité de la
   société (pour réparer #528) **sans** vider toutes ses tables rattacherait à la société remise à zéro
   les factures, contacts, taux et réglages de la démonstration — et `finalize` trouverait des
   réglages de facturation pointant des comptes effacés. Les deux se ferment ensemble (C-15-7-10).
7. **L'ordre des `DELETE` canonique inverse le Pattern 5** : `TABLES_TO_TRUNCATE` va des enfants aux
   parents — `company_invoice_settings` et `bank_accounts` avant `accounts`, les séquences et
   `journal_entries` avant `fiscal_years` — alors que le Pattern 5 prend `accounts` avant
   `company_invoice_settings`, et que `invoices::validate_invoice` (qui ne prend pas `companies`)
   verrouille `fiscal_years` avant `invoice_number_sequences` et `journal_entries`. Un interblocage
   avec une transaction de ce genre est possible ; d'où C-15-7-22.
8. **Aucun changement de schéma** : ni migration, ni P1–P8.

## Acceptance Criteria

### Volet B — la remise à zéro

**2. Une seule transaction, les trois gardes sous son verrou, et un rejeu sur interblocage**
(C-15-7-5, C-15-7-11, C-15-7-22). `kesh_seed::reset_demo(pool, actor, production_reset_allowed: bool)
-> Result<ResetOutcome, SeedError>`, où **`pub struct ResetOutcome { pub company_id: i64, pub
company_recreated: bool, pub audit_entries_erased: u64, pub principals: OrphanPrincipals }`**
(`OrphanPrincipals`, AC 4) — champs **publics** : le test 8 vit dans le crate de tests de `kesh-api`
(R4-4 de la P4) — reprend les champs homonymes de l'entrée `installation.reset` (AC 5) ; le handler ne
s'en sert pas (il relit l'état), le test 8, qui appelle `reset_demo` directement, l'asserte. Le handler
**garde** `get_or_init_state` **avant** l'appel (`routes/onboarding.rs:244` — sans lui, une ligne
absente ferait rendre `None` au verrou) et, **après**, la relecture de l'état et `response_with_stub`
(`:293-295`) ; il lit le drapeau **une fois** (`env_flag_enabled("KESH_PRODUCTION_RESET")`) et le
passe ; il **n'ouvre plus de transaction** et ne porte plus de garde : la seule copie des gardes est
dans `reset_demo`, évaluée sur l'état verrouillé **par la transaction qui efface**. Le corps entier est
enveloppé dans `kesh_db::retry::retry_with` (`DEFAULT_MAX_DEADLOCK_ATTEMPTS`, patron `finalize`,
`routes/onboarding.rs:606-614`). ⚠️ **Le prédicat doit voir les erreurs de l'essai** :
`is_deadlock_error` (`kesh-db/src/retry.rs:82-84`) ne reconnaît que `DbError::Sqlx`, alors qu'un `?`
brut sur `sqlx::Error` produirait `SeedError::Sqlx` (`#[from]`, `kesh-seed/src/lib.rs:19-20`) — le
rejeu serait du code mort, en silence. D'où : le corps d'un essai est une fonction privée qui rend
**`Result<ResetOutcome, SeedAttemptError>`** — le type que pose la 15-7b1 pour la dernière transaction
de `seed_demo` (`#[doc(hidden)] pub enum SeedAttemptError { Db(DbError), StepAlreadyCompleted }`),
**étendu ici** d'une variante `ResetForbidden` (C-15-7-38 ; R2-6 de la P2 : un seul type d'essai, un
seul prédicat pour les deux flux), avec `From<DbError>` et **sans** `From<sqlx::Error>` :
**toute** erreur sqlx de l'essai — `acquire`, `begin`, `SET`, `SELECT … FOR UPDATE`, chaque `DELETE`,
`commit` compris — passe **par `map_db_error`**, faute de quoi le code ne compile pas. Le prédicat est
la fonction `#[doc(hidden)] pub fn is_seed_retryable(e: &SeedAttemptError) -> bool { matches!(e,
SeedAttemptError::Db(d) if is_deadlock_error(d)) }`, posée par la 15-7b1 et **réemployée** ; après
`retry_with`, `SeedAttemptError` se convertit en `SeedError` (`Db`, `StepAlreadyCompleted`,
`ResetForbidden`). Le prédicat est exercé par le
test 13 sur une **vraie** erreur 1213 ; **chaque essai** acquiert sa connexion dédiée, la marque
**`close_on_drop()`** aussitôt (AC 6), ouvre la transaction, et la referme — un essai désigné victime
d'un interblocage (1213) est annulé en entier par InnoDB et rejoué de zéro, gardes comprises. Dans un
essai, sur la connexion dédiée (`pool.acquire()`, transaction par `sqlx::Connection::begin(&mut
*conn)`, `SET FOREIGN_KEY_CHECKS=0` **dans** la transaction, comme `restore_tables_in_tx`,
`backup.rs:405-431`), `reset_demo` :
  1. `onboarding::lock_state_in_tx` (15-7a1) — `None` ⇒ `Db(DbError::Invariant(…))`, 500, rien
     d'effacé (le handler a garanti la ligne par `get_or_init_state` : son absence est une corruption ;
     sans ce cas, `reset_state_in_tx` ne toucherait aucune ligne en silence ; **garde inatteignable par
     HTTP** (L-4 de la P2) : si la ligne a disparu, le handler la **recrée à l'étape 0, non
     démonstration**, par `get_or_init_state`, et la remise à zéro y passe sans drapeau — une installation
     de production dont la ligne aurait disparu serait vidée (33 tables). Préexistant, portée élargie par
     #279 ; angle mort écrit au Dev Agent Record, non corrigé ici) — et applique, **dans cet
     ordre** (celui du handler actuel) : `step_completed >= 7` ⇒ `SeedError::StepAlreadyCompleted`
     (la variante que pose la 15-7b1, même code HTTP — R2-6 de la P2 : la variante `AlreadyFinalized`
     de la rédaction précédente faisait doublon) ;
     `!is_demo && step_completed > 2` ⇒ `SeedError::ResetForbidden` ; `step_completed > 2 &&
     !production_reset_allowed` ⇒ `SeedError::ResetForbidden` (portés par `SeedAttemptError` dans
     l'essai). Le handler rend `StepAlreadyCompleted` en **`400 ONBOARDING_STEP_ALREADY_COMPLETED`**
     (`errors.rs:1345-1352`) et `ResetForbidden` en **`403 ONBOARDING_RESET_FORBIDDEN`**
     (`:1354-1360`) — les codes d'aujourd'hui. **Correspondance complète `SeedError` → `AppError`**
     (R5 de la P1 ; énumère les **quatre** variantes de `SeedError` une fois cette fiche livrée — deux
     d'origine, `Db` et `Sqlx` (`kesh-seed/src/lib.rs:14-21`), `StepAlreadyCompleted` posée par la 15-7b1,
     `ResetForbidden` ici ; R2-6 de la P2, R3-2 de la P3, R4-3 de la P4),
     écrite dans le handler : `StepAlreadyCompleted` ⇒ 400 ci-dessus ; `ResetForbidden` ⇒
     403 ci-dessus ; `Db(d)` ⇒ **`AppError::Database(d)`** (`Invariant` ⇒ 500 `INTERNAL_ERROR`, message
     journalisé, jamais rendu au client, `errors.rs:3294-3301` ; un 1213 épuisé après les essais de
     `retry_with` ⇒ `Database(Sqlx)`, 500 journalisé) ; `Sqlx(e)` (inatteignable par le chemin de
     `reset_demo`, toute erreur passant par `map_db_error`) ⇒ `AppError::Internal`, 500 générique
     (`errors.rs:1313-1320`). L'aplatissement actuel en `Internal(format!("Reset demo failed: {e}"))`
     (`routes/onboarding.rs:289-291`) disparaît ;
  2. verrouille `companies` `FOR UPDATE ORDER BY id`. **Une** société ⇒ elle est conservée (AC 4) ;
     **aucune** ⇒ le stub est **inséré** dans la transaction (`companies::insert_stub`, AC 4) — c'est
     l'état d'une installation déjà atteinte par #528 sous v0.12.x, qui doit pouvoir se réparer par une
     nouvelle remise à zéro au lieu de rendre 500 ; **plus d'une** ⇒ `DbError::Invariant`, 500, rien
     d'effacé (même invariant que `seed_demo`, `lib.rs:105-111`). ⚠️ **Les installations déjà touchées
     par #542** (plusieurs sociétés provisoires) **ne sont plus réparées ici** (F4-3 de la P4,
     C-15-7-46) : l'état visé n'atteint jamais le bouton de remise à zéro (il vit dans le bandeau de
     démonstration, et `seed_demo` y rend 500) ; la **15-7b3** supprime les sociétés provisoires
     superflues **au démarrage**. Cette fiche garde la prévention (AC 13) ;
  3. relève `COUNT(*)`, `MIN(id)`, `MAX(id)` d'`audit_log` **par une lecture verrouillante**
     (`… FROM audit_log FOR UPDATE`) ;
  4. vide les tables de `reset_cleared_tables()` (AC 3) dans l'ordre de `TABLES_TO_TRUNCATE` ; le
     `rows_affected` du `DELETE FROM audit_log` doit égaler le `COUNT(*)` relevé (sinon `Invariant`) —
     garde défensive **non testée, angle mort assumé** (R6 de la P1). Qu'aucune insertion ne puisse
     s'intercaler sous la lecture verrouillante de l'étape 3 est une **conjecture**, non vérifiée contre
     MariaDB (F3-6 de la P3) ; la sûreté n'en dépend pas — le `DELETE` verrouille lui-même ce qu'il
     efface, et un écart rend `Invariant`, rien d'effacé ;
  5. remet la société à l'état stub **en place** (si elle existait) et traite les principaux orphelins
     par `reattach_orphan_principals_in_tx(tx, Some(company_id))` — utilisateurs rattachés, clés d'API
     actives orphelines **révoquées puis** repointées (AC 4) ;
  6. remet `onboarding_state` à zéro **en place** : `onboarding::reset_state_in_tx` (neuf) —
     `UPDATE … SET step_completed = 0, is_demo = FALSE, ui_mode = NULL, version = version + 1`
     (C-15-7-16). `delete_state` perd son seul appelant de production (`kesh-seed/src/lib.rs:303`) ; il
     reste pour son test (`crates/kesh-db/tests/onboarding_repository.rs:83`), son doc-comment
     (`repositories/onboarding.rs:132-135`) est corrigé (AC 10). `init_state` garde ses appelants
     (`get_or_init_state`, tests) ;
  7. écrit **en dernier** `installation.reset` (AC 5) ; `SET FOREIGN_KEY_CHECKS=1` ; `COMMIT`.

⚠️ **Ordre des verrous, complet** (L-1 de la P4) : `onboarding_state → companies → audit_log → tables
de reset_cleared_tables() dans l'ordre de TABLES_TO_TRUNCATE → users → api_keys`. (L'ordre conditionnel
de la suppression des sociétés superflues — `users`, `api_keys` en partagé après `companies`, R2-4/L-1 de
la P2 — a quitté cette fiche avec elle, C-15-7-46.) Les deux derniers par les `SELECT … FOR UPDATE` puis
les `UPDATE` de `reattach_orphan_principals_in_tx` (étape 5, AC 4), qui posent des verrous exclusifs
sur les lignes lues ; l'entrée d'audit relit ensuite `users` par
sous-SELECT (`repositories/audit_log.rs:91-92`). Les deux premiers suivent le Pattern 5 ; la suite le
contredit (inventaire § 7). `reset_demo` est donc inscrite à la **liste d'exceptions** du Pattern 5
(AC 10), avec ce qui l'atténue : les deux verrous de tête sérialisent toute transaction qui suit le
Pattern 5 avant qu'elle n'atteigne les tables inversées ; le cycle n'est possible qu'avec une
transaction qui ne prend pas `companies` (`invoices::validate_invoice`, un écrivain d'audit tenant déjà
une ligne métier, `users::update_role_and_active_in_tx` qui tient `users` puis insère dans
`audit_log`) ; la victime désignée par InnoDB est annulée **en entier**, puis rejouée.

**3. La remise à zéro vide toute donnée de la société, par la liste canonique** (C-15-7-10, #279).
`kesh_db::backup` — à côté de la liste dont elles dérivent et de la garde
`backup_inventory_matches_schema` — définit `RESET_PRESERVED_TABLES = ["api_keys", "companies",
"onboarding_state", "password_reset_tokens", "refresh_tokens", "users"]` ; `reset_cleared_tables()` est
**dérivé** : `TABLES_TO_TRUNCATE` privé de ces six, **dans l'ordre** de `TABLES_TO_TRUNCATE`
(`audit_log` inclus ; 39 − 6 = **33** tables à ce jour). **Forme fixée** (R2 de la P1 — une `const` ne
peut pas filtrer une autre `const` en tranche de longueur inconnue) : une **fonction**
`pub fn reset_cleared_tables() -> Vec<&'static str>` (`TABLES_TO_TRUNCATE.iter().copied().filter(|t|
!RESET_PRESERVED_TABLES.contains(t)).collect()`), appelée une fois par essai ; pas de `LazyLock`, le
coût d'un filtre de 39 noms est nul pour un geste rare. Tous les sites de la fiche l'emploient sous ce
nom. Aucune liste de tables n'est plus écrite en dur
dans `reset_demo`. **La partition est gardée sur le schéma** (C-15-7-24, amendé par C-15-7-28), par un
test `#[sqlx::test]` qui lit `information_schema.KEY_COLUMN_USAGE` (`TABLE_SCHEMA = DATABASE()`,
`REFERENCED_TABLE_NAME IS NOT NULL`). **Relation employée** : une table *rejoint* `companies` s'il
existe une chaîne de clés étrangères qui y mène **sans traverser aucune table de
`RESET_PRESERVED_TABLES` autre que `companies`** — en particulier **sans passer par `users`**. Une
fermeture transitive ordinaire ne garderait rien : `refresh_tokens → users → companies` y « atteint »
`companies`, si bien qu'une future table enfant de `users` seulement serait classée « à vider » sans
que rien ne rougisse.
  - **Règle 1** : toute table de `reset_cleared_tables()` rejoint `companies`, à une exception près,
    **nommée dans une liste fermée** du test : `audit_log`, qui n'a **aucune** clé étrangère
    (`company_id` est un pointeur logique sans FK, et `fk_audit_log_user` a été retirée par la migration
    `20260910000001`) ; effacée et réinscrite à dessein (AC 5). Le test vérifie que chaque exception de
    la liste **ne rejoint effectivement pas** `companies` (une exception devenue inutile rougit).
  - **Règle 2** : toute table de `TABLES_TO_TRUNCATE` qui porte une clé étrangère vers une table
    préservée autre que `companies` (`users`, `api_keys`…) **et** ne rejoint pas `companies`
    appartient à `RESET_PRESERVED_TABLES` — aujourd'hui exactement `password_reset_tokens` et
    `refresh_tokens` (asserté en dur). Une future table de seconds facteurs ou de préférences, enfant
    de `users` seulement, fait rougir le test et force le choix au lieu d'être vidée en silence.
  - **Règle 3** (F-3 de la P4) : **aucune** clé étrangère d'une table de `RESET_PRESERVED_TABLES` ne
    référence une table de `reset_cleared_tables()` (ensemble vide asserté). Le vidage se fait sous
    `FOREIGN_KEY_CHECKS=0` : une telle clé — `users.default_project_id`, `companies.logo_document_id`,
    `api_keys.account_id`, colonnes futures imaginables — laisserait une **référence pendante** sans
    1451 ni cascade. Aujourd'hui vrai (graphe du squash : `api_keys → companies, users` ; `users →
    companies` ; `password_reset_tokens`, `refresh_tokens → users` ; `companies`, `onboarding_state` :
    aucune).
  - **Angle mort assumé** (L-6 de la P4) : une future table enfant de `users` **et** de `companies`
    (préférences par utilisateur portant un `company_id`) rejoint `companies` directement ; la règle 2
    ne la voit pas, elle est classée « à vider ». C'est le comportement voulu pour des données de
    société ; si elle porte des réglages d'utilisateur à conserver, le choix se fait à sa création.
Recompté sur le squash le 2026-10-08 **avec cette relation** (script de lecture des `REFERENCES`,
revérifié par les deux lentilles de la P4) : 32 des 33 tables vidées rejoignent `companies` sans
traverser une table préservée ; `audit_log` est la seule exception ; la règle 2 désigne
`password_reset_tokens` et `refresh_tokens` ; la règle 3 est vide. ⚠️ Vérifier, table par table,
qu'une table vidée qui porte des réglages par défaut est recréée à la demande pour une société
existante (`company_dunning_settings` est get-or-create, `repositories/company_dunning_settings.rs:52` ;
`dunning_levels` est paresseux, gardé par `seeded_at`) ou par `finalize` (`company_invoice_settings`,
`vat_rates`) ; tout cas contraire se signale au Dev Agent Record — la remise à zéro doit laisser
l'état d'une société neuve.

**4. L'identité de la société est préservée ; les principaux orphelins sont traités par une règle et
une fonction uniques** (C-15-7-9, C-15-7-23, C-15-7-45, #528). Trois fonctions `kesh-db`,
`repositories/companies.rs` :
- `companies::reset_to_stub_in_tx(tx, id)` écrit `name = STUB_COMPANY_NAME`,
  `address = STUB_COMPANY_ADDRESS`, `org_type = OrgType::Independant`, `accounting_language =
  Language::Fr`, `instance_language = Language::Fr`, `is_stub = TRUE`, `version = version + 1`, et
  ramène chaque autre colonne à **la valeur qu'elle prend à l'insertion du stub**, c'est-à-dire à son
  défaut du schéma (`crates/kesh-db/test-schema/0001_schema_squash.sql`, table `companies`, vérifié
  le 2026-10-08) :
  - **`NULL`** pour les **sept** colonnes nullables : `first_name`, `last_name`, `ide_number`, `email`,
    `phone`, `website`, `books_locked_through` ;
  - **`''`** pour `address_street`, `address_building`, `address_postal_code`, `address_city`
    (`NOT NULL DEFAULT ''` — un `NULL` y rendrait l'erreur 1048) ;
  - **`'CH'`** pour `address_country` et `country` (`NOT NULL DEFAULT 'CH'` ; aucun code n'écrit
    `country` aujourd'hui) ;
  cette énumération est **ouverte** par nature ; ce qui la ferme est le test 5, qui compare **toutes**
  les colonnes lues dans `information_schema.COLUMNS` (F-4 de la P4) ;
- `companies::insert_stub(executor, instance_language) -> Result<i64, DbError>` (générique sur
  l'exécuteur) : l'`INSERT` du stub, **seul** site d'insertion — `reset_demo` lui passe
  **`Language::Fr`**, la langue qu'écrit `reset_to_stub_in_tx` et que pose le bootstrap
  (`auth/bootstrap.rs:356-357`), de sorte qu'une société recréée et une société remise à zéro soient
  identiques ; la branche « aucune société » passe la langue demandée. Appelé par le bootstrap
  (`auth/bootstrap.rs:347-360`, `insert_stub_company` devient une enveloppe), par la branche « aucune
  société » d'`ensure_company_with_language` (`routes/onboarding.rs:865-883`) et par `reset_demo`
  (AC 2.2) ;
- **`companies::reattach_orphan_principals_in_tx(tx, target: Option<i64>) -> Result<OrphanPrincipals,
  DbError>`** — la règle unique des principaux orphelins (C-15-7-45 ; remplace
  `attach_all_principals_in_tx` et son repointage nu des clés, R4-1/F4-1 de la P4), avec
  `pub struct RevokedApiKey { pub id: i64, pub name: String, pub created_by_user_id: i64, pub created_at,
  pub last_used_at }` (types de l'entité `ApiKey` ; **aucun** préfixe : `api_keys` ne porte que
  l'empreinte `key_hash`, jamais rendue) et `#[derive(Default)] pub struct OrphanPrincipals { pub
  user_ids: Vec<i64>, pub api_keys_revoked: Vec<RevokedApiKey>, pub api_keys_repointed: u64 }`
  (`is_empty()` : les trois vides). Est **orpheline** une ligne dont le `company_id` ne désigne aucune
  société (`NOT EXISTS (SELECT 1 FROM companies c WHERE c.id = <t>.company_id)`). Pré-condition :
  l'appelant tient `companies` (Pattern 5) et `target`, s'il est donné, existe. Dans cet ordre :
  1. `SELECT id FROM users WHERE <orpheline> ORDER BY id FOR UPDATE` ;
  2. `SELECT id, company_id, version, name, created_by_user_id, created_at, last_used_at FROM api_keys
     WHERE revoked_at IS NULL AND <orpheline> ORDER BY id FOR UPDATE` — les clés **actives** orphelines ;
  3. **révoque** chacune par `api_keys::revoke_in_tx(tx, key.company_id, key.id, key.version)` (le
     `company_id` mort de la clé ; un `OptimisticLockConflict` est impossible sous le verrou de l'étape 2,
     et remonte s'il survient) — **quel que soit `target`** ;
  4. si `target = Some(c)` : `UPDATE users SET company_id = c WHERE <orpheline>`, `rows_affected` égal à
     la longueur de la liste 1, sinon `DbError::Invariant` (`user_ids` = cette liste) ; puis `UPDATE
     api_keys SET company_id = c WHERE <orpheline>` — **toutes** les clés orphelines, révoquées à
     l'étape 3 ou déjà révoquées avant — qui ne touche que `company_id` (`version` et `revoked_at`
     inchangés) ; `api_keys_repointed` = son `rows_affected`. Si `target = None` : ni rattachement ni
     repointage (`user_ids` vide, `api_keys_repointed = 0`) — les clés restent révoquées au `company_id`
     mort, repointées plus tard par le même appel.
  Ainsi **aucune clé orpheline ne redevient active par aucun chemin** : une clé repointée l'est toujours
  révoquée, et paraît, sous son nom et au statut « révoquée », sur la page des clés de la société
  (`list_by_company`, `include_revoked = true`). Avec une seule société, « orpheline » et « désigne une
  autre société » (`WHERE NOT (company_id <=> ?)`, rédaction précédente) coïncident ; le prédicat
  d'orphelin est retenu parce que la 15-7b3 appelle aussi la fonction sur une base à zéro ou plusieurs
  sociétés. Appelée par `reset_demo` (étape 5, `Some`) **et** par la branche « aucune société »
  d'`ensure_company_with_language`, juste après l'`INSERT` (`Some` de la société créée) — qui ne recrée
  donc plus jamais une société sans rattacher ; et par la **15-7b3** au démarrage et à la restauration.
  Cette branche ajoute `"users_repointed"`, `"api_keys_revoked"` (liste des `RevokedApiKey`, sans
  empreinte) et `"api_keys_repointed"` aux `details` de `company.created` (15-7a2).
Les constantes `STUB_COMPANY_NAME` / `STUB_COMPANY_ADDRESS` (`auth/bootstrap.rs:36-37`) **descendent**
dans `kesh-db` (`kesh-seed` ne dépend pas de `kesh-api`) ; leur doc-comment (`:31-35`) suit (R4-7 de la
P4). Conséquences : après une remise à zéro **d'une installation saine** (`users.company_id` désignait
la société conservée), le JWT en cours reste valide, `GET /api/v1/companies/current` répond **200**,
et `language` n'emprunte plus le chemin « aucune société » — il écrit `company.updated
{instance_language}` si la langue choisie n'est pas `fr`, rien sinon. **La réponse de `reset` porte
désormais `isStub: true`** (`response_with_stub`, `routes/onboarding.rs:85-98`, lit la société
conservée ; elle portait `false` quand la société était effacée) : le bandeau
`onboarding-stub-notice` (`frontend/src/routes/onboarding/+layout.svelte:16-29`) s'affiche dès l'étape
0 après une remise à zéro — l'état d'une installation neuve, voulu.
⚠️ **Limite — installation déjà atteinte par #528** : la réparation corrige la colonne
`users.company_id`, **pas le jeton déjà émis**, qui porte le `company_id` du moment de sa création
(`middleware/auth.rs:159`, péremption acceptée et documentée `:64-72`). Jusqu'à son renouvellement —
`refresh` (`routes/auth.rs:437`, `:539` relisent la colonne), au plus `KESH_JWT_EXPIRY_MINUTES`
(défaut 15 min, `config.rs:673`) **plus 60 s de tolérance** (`middleware/auth.rs:60-61`, R4-9 de la
P4) — ou à une reconnexion, les routes qui passent par `get_company_for` répondent encore **500**
(`helpers.rs:61-72`), celles qui lisent `current_user.company_id` directement rendent des listes vides,
ou une erreur de clé étrangère à l'écriture (F3-4 de la P3). Cela vaut
pour les cas 6a, 6b et 6c, et pour la réparation de la 15-7b3 ; le manuel le dit (AC 11) ;
aucun test n'asserte de 200 sur une route scopée avec l'**ancien** jeton dans ces cas.

**5. L'entrée `installation.reset`, et ce qu'elle dit** : `entity_type = "installation"`,
`entity_id = AUDIT_ENTITY_ID_NONE`, `details = {"step_before", "is_demo_before", "company_id",
"company_recreated", "audit_entries_erased", "erased_id_min", "erased_id_max", "tables_cleared": [...],
"users_repointed", "api_keys_revoked": [{"id", "name", "created_by_user_id", "created_at",
"last_used_at"}], "api_keys_repointed"}` — `company_recreated` vrai ssi le stub a été inséré (base sans
société) ; `erased_id_*` à `null` si la piste était vide ; `tables_cleared` = `reset_cleared_tables()`
**exactement** (la piste n'affirme que ce qui a été vidé) ; `users_repointed` = la longueur de
`user_ids`, `api_keys_*` = ce que rend `reattach_orphan_principals_in_tx` — les **champs `api_keys_*`** ont la
forme de ceux de l'entrée `installation.repaired` de la 15-7b3 (C-15-7-49), qui porte **en plus**
`user_ids` ; l'entrée `installation.reset` n'en porte pas (R5-4 de la P5). Après une remise à zéro réussie, **`audit_log` contient
exactement une ligne**, `installation.reset`, son `company_id` (sous-SELECT sur `users`) est celui de la
société conservée ou recréée — l'entrée s'écrit **après** le rattachement —, et son `id` est
**supérieur** à `erased_id_max` (auto-incrément persistant sous MariaDB ≥ 10.2.4 ; le test l'asserte).
Le trou dans la numérotation s'explique par l'entrée qui le suit.

**6. Un échec à n'importe quel point ne laisse rien effacé, et la connexion ne retourne jamais au
pool** (C-15-7-25, C-15-7-32, F-2 de la P4). Chaque essai appelle **`conn.close_on_drop()` juste après
`pool.acquire()`** (sqlx-core 0.8.6 : `PoolConnection::close_on_drop`, `pool/connection.rs:97` ; son
`Drop`, `:199-207`, ferme au lieu de rendre) : la connexion qui a porté `FOREIGN_KEY_CHECKS=0` est
**fermée** à sa libération, sur succès, sur erreur **et sur abandon de la future** (déconnexion du
client — hyper abandonne alors la future du handler —, arrêt du serveur, délai), cas que la rédaction
précédente ne couvrait pas : la `Transaction` lâchée annulait bien, mais la connexion retournait au
pool avec les contrôles désactivés. Les contrôles FK sont en outre rétablis **avant** le `COMMIT`
(`SET FOREIGN_KEY_CHECKS=1` en étape 7), sur le patron de `restore_tables_in_tx`
(`backup.rs:405-431`) : un `SET` qui échoue fait annuler la transaction et rendre l'erreur, rien
n'est effacé. Sur toute erreur : `tx.rollback().await` **explicite**, puis l'erreur **d'origine** —
l'erreur éventuelle du rollback est journalisée (`tracing::warn!`) et **jamais** substituée à
l'erreur d'origine (R7 de la P1) : après un 1213, InnoDB a déjà annulé la transaction, le `ROLLBACK`
peut échouer à son tour, et s'il remplaçait le 1213, `is_seed_retryable` ne verrait plus
l'interblocage et le rejeu n'aurait pas lieu (même règle que le rollback *best-effort* de l'AC 1 de la
15-7a1). **Ni** `SET` de
rétablissement sur le chemin d'erreur, **ni** `PoolConnection::detach()` : la fermeture les rend sans
objet (et `detach()` laissait dépasser `max_connections`, L-5 de la P4). Le coût — une connexion
rouverte par remise à zéro — est négligeable pour un geste rare. ⚠️ La même faille dans
`restore_tables_in_tx` (restauration de sauvegarde) relève de **#540**, hors périmètre.

### Volet C — les invariants

**7. Le constructeur dit la vérité sur l'acteur** : `reset_demo` écrit par `for_actor(user_id,
api_key_id, …)` (posé par la 15-7b1), acteur threadé depuis `Extension(current_user)` du handler
`reset`. La garde de source de la 15-7b1 (test 11) couvre `kesh-seed/src/lib.rs` entier.

**8. Le registre des routes passe `reset` à `Traced`** (`audit_route_registry.rs:76`) ; partition
**recomptée depuis la source** : sur la base de la 15-7b1 mergée, `traced` 105 → **106**, `exempt` 5 →
**4** (les quatre de #435), `no_matter` **2**, total **112** *(corrigé au T0 de la 15-7b1 : la 15-7a2 a
livré 104 / 6 / 2 et non 103 / 6 / 3, la 15-7b1 laisse 105 / 5 / 2 — à recompter encore au T0 de cette
fiche)*. Le message « 1 route d'onboarding (#434,
15-7b2) » disparaît ; le message de l'assertion **`traced`** (`:469-475`) reçoit « plus la remise à zéro
(15-7b2, #434) » (propagation de R2-6 de la P2 de la 15-7b1). ⚠️ *Recompter au merge.*

**9. Une action neuve, déclarée et libellée dans les quatre catalogues** (la seconde de la P2,
`installation.principals_reattached`, est partie avec l'AC 14 à la 15-7b3 — C-15-7-40 —, où elle est
devenue `installation.repaired` — C-15-7-47) (`ACTIONS` triée, quatre
`.ftl` près de `audit-log-action-installation-demo-seeded`, posée par la 15-7b1) :

| Action | Clé | fr-CH | de-CH | it-CH | en-CH |
|---|---|---|---|---|---|
| `installation.reset` | `audit-log-action-installation-reset` | Installation réinitialisée | Installation zurückgesetzt | Installazione reimpostata | Installation reset |

**10. Les doc-comments que la story rend faux disent le nouvel état, et le Pattern 5 porte
l'exception.** Grep exécuté, sortie collée au Dev Agent Record :
`grep -rnE "#434|#279|reset_demo|DELETE FROM audit_log|KF-002-H-002|lock-and-release|gate-check only|rare hors bootstrap|is_stub|placeholder|second des trois chemins|Deny list|\(none\)|FK RESTRICT de .users.company_id|KESH.{1,2}PRODUCTION.{1,2}RESET" crates docs/MULTI-TENANT-SCOPING-PATTERNS.md .env.example docker-compose.yml docker-compose.prod.yml docker-compose.dev.yml frontend/src`
(les deux compose de production y figurent depuis la P5, F5-1 : une fois la 15-11a mergée, le grep doit
y rendre la transmission de `KESH_PRODUCTION_RESET` — son absence est un défaut de dépendance, non un
site à corriger ici, C-15-7-51)
(`is_stub` rend de nombreuses occurrences légitimes — colonnes, tests —, à trier à la main ; un site du
grep absent de la liste se traite quand même). Sites **attribués à cette fiche** (attribution complète
dans l'index) :
- `kesh-seed/src/lib.rs` — doc de `reset_demo` (`:228-233`) ;
- `kesh-db/src/repositories/audit_log.rs:16-23` — chemin 2 : la remise à zéro efface **et inscrit**
  `installation.reset` ;
- `kesh-api/src/lib.rs:302-311` — commentaire de la route `reset` (« Elle efface `audit_log` … second
  des trois chemins ») : elle efface **et inscrit** son geste, en une transaction qui porte les gardes ;
- `kesh-api/src/routes/onboarding.rs:62` (doc d'`is_stub` : « créée par le bootstrap » — aussi par la
  remise à zéro), `:225-231` (doc de `reset` : « in production deployments it must remain unset; only
  demo deployments set it » — faux depuis C-15-7-48 : sortir d'une démonstration exige le drapeau, posé
  le temps de la réinitialisation puis retiré ; R5-2 de la P5), `:232-239` (doc de `reset` : verrou relâché → transaction unique), `:281-293`,
  `:866-869` (« rare hors bootstrap » : désormais une base sans société, avec rattachement) ;
- `kesh-api/src/errors.rs:282` (doc d'`OnboardingResetForbidden` : « production sans
  `KESH_PRODUCTION_RESET=1` » — dire : drapeau absent ; R5-2 de la P5) ;
- `kesh-db/src/entities/company.rs:174-179` — `is_stub` : le stub est aussi recréé **en place** par la
  remise à zéro, et inséré par elle sur une base sans société ;
- `kesh-db/src/repositories/onboarding.rs:132-135` (`delete_state` : « l'orchestration complète du
  reset est dans `reset_demo` » — plus vrai) ;
- `kesh-db/src/repositories/accounts.rs:1020-1021` (« utilisé par reset_demo » : relire) ;
- `kesh-api/src/auth/bootstrap.rs:31-37` — constantes descendues, doc qui suit ; doc de
  `ensure_admin_user` et commentaire du cas 1 (`:72-73`) : le stub n'est inséré que sur une base sans
  société (AC 13) ; la réparation de #528 au démarrage, et sa doc, relèvent de la 15-7b3 ;
- `frontend/src/routes/onboarding/+layout.svelte:17-19` — commentaire « la company a été créée en
  placeholder par le bootstrap (DB vide) » : aussi par la remise à zéro, en place ou insérée (L-5 de la
  P2 ; le motif du grep ci-dessus porte aussi sur `frontend/src`) ;
- `kesh-api/src/helpers.rs:55-58` — doc de `get_company_for` : « ce qui ne devrait jamais arriver grâce
  à la FK RESTRICT de `users.company_id` » est **faux** (F-5 de la P1) : `fk_users_company` est
  `ON DELETE CASCADE` (squash, table `users`), et le `company_id` orphelin est précisément le cas #528 ;
  le doc dit : un jeton émis avant une réparation de #528 peut porter un id mort jusqu'à son
  renouvellement (AC 4, « Limite ») ;
- `.env.example:273-279` (« … pourrait alors wipe une instance déployée » ; « Valeurs acceptées … "1",
  "true", "yes" » omet `on`, que `env_flag_enabled` accepte, `routes/onboarding.rs:45-53`) et
  `docker-compose.dev.yml:28-29` (commentaire de `KESH_PRODUCTION_RESET`) — révélés par le motif corrigé
  de l'AC 11 (F-4 de la P1) : la remise à zéro vide désormais **toutes** les données de la société et en
  conserve utilisateurs et clés d'API ; une installation de production n'est jamais remise à zéro ;
  **sortir d'une démonstration exige ce drapeau** (la démonstration est à l'étape 3) — à poser le temps
  de la réinitialisation, puis à retirer (C-15-7-48) ;
- `docs/MULTI-TENANT-SCOPING-PATTERNS.md:313` (ligne `reset` de la table ; `:314` est la ligne
  `seed_demo`, à la 15-7b1 — R3-5 de la P3) : l'ordre réel complet
  `onboarding_state → companies → audit_log → reset_cleared_tables() (ordre de TABLES_TO_TRUNCATE) →
  users → api_keys` (L-1 de la P4 ; l'ordre conditionnel de la suppression des sociétés superflues a
  quitté cette fiche avec elle, C-15-7-46) ; `:320-330` (« Known Risk — KF-002-H-002 ») : la remise à zéro
  n'est plus en lock-and-release (`seed_demo` le reste pour ses quatre premières validations — renvoi à
  **#538**, posé par la 15-7b1) ; et la **liste d'exceptions** (« Deny list », `:330-336`, aujourd'hui
  `*(none)*`) reçoit une ligne `kesh_seed::reset_demo` — motif : vidage dans l'ordre FK enfants →
  parents, imposé par la liste canonique ; atténuation : verrous `onboarding_state` puis `companies`
  pris en premier, victime d'un interblocage annulée en entier puis rejouée par `retry_with`
  (C-15-7-22).
Sites légitimes à laisser : `kesh-db/src/test_fixtures.rs:176` ; `journal_entries.rs:1189-1190`
(historique) ; `retry.rs:7`, `reconciliation.rs:3831`.

### Volet D — ce que lisent l'utilisateur et l'administrateur

**11. Les manuels disent ce que l'inventaire établit**, PDF régénérés (`make fr`) et contrôlés
aplatis :

| Site | Après la 15-7b1 | Après la 15-7b2 |
|---|---|---|
| `admin-manual.tex:1948` (`:1821` avant la 15-7a2) | « 105 des 112 routes » ; exceptions \#434 (la remise à zéro) et \#435 | « 106 des 112 routes » ; seule exception : les gestes de session (\#435), plus les deux routes « sans matière » (106 + 4 + 2 = 112) — *recompte du T0 de la 15-7b1* |
| `admin-manual.tex:1845-1860` (réserve OLICo) | « La réinitialisation des données de démonstration efface encore la table » | elle l'efface **et y inscrit son geste** (`installation.reset` : nombre et plage d'identifiants effacés) ; toujours réservée à l'administrateur et refusée après finalisation |
| `admin-manual.tex:2004` | la remise à zéro (\#434) et les gestes de session (\#435) | seuls les gestes de session (\#435) |
| `admin-manual.tex:691` (`KESH_PRODUCTION_RESET`, L-3 de la P4) | « Autorise le reset d'onboarding au-delà de l'étape 2 » | « Autorise la réinitialisation d'une **démonstration** au-delà de l'étape 2 — donc **toute** sortie de la démonstration, qui est à l'étape 3 (une installation de production ne l'est jamais) ; à poser le temps de la réinitialisation, puis à retirer, chaque fois suivie de `docker compose up -d` (`restart` ne relit pas `.env`) ; la réinitialisation vide alors **toutes** les données de la société » (valeurs `1`, `true`, `yes`, `on`, sans casse — `routes/onboarding.rs:45-53`). ⛔ **La mise en page des dix tableaux de `sec:env-vars` n'est plus à faire ici** : elle est **faite par la 15-11a, AC12 (j)** (C-15-7-54, qui révise C-15-7-52) ; la 15-11a merge avant cette story et ne touche ni le texte des titres `\paragraph{…}`, ni cette ligne. **Vérifier après rebase** que la cellule `KESH\_PRODUCTION\_RESET` se lit **entière** dans le PDF aplati (`pdftotext -layout` : la phrase réécrite ci-dessus, de « Autorise » à « données de la société », sans rognage), la ligne se relocalisant par le texte. La recette « poser la variable » suppose la **15-11a mergée** (C-15-7-51) |
| `admin-manual.tex:1314` (« Parcours ») | — | **sortir de la démonstration** (F4-2/R4-2 de la P4, C-15-7-48) : la seule voie est la réinitialisation (bouton « Réinitialiser pour la production » du bandeau de démonstration), et elle n'est permise que si l'exploitant l'a autorisée — poser `KESH_PRODUCTION_RESET` (`true` ; `1`, `yes`, `on` aussi) dans le fichier `.env`, puis `docker compose up -d` (et non « redémarrer » ni `docker compose restart` : `restart` ne relit pas `.env`, la variable n'atteindrait pas le conteneur et le bouton resterait refusé), réinitialiser, puis **retirer** la variable et refaire `docker compose up -d` (recette exécutable une fois la **15-11a** mergée, qui transmet la variable au conteneur — #550, C-15-7-51 ; d'où l'ordre de merge imposé en tête de fiche) ; sans elle, le bouton est **refusé (403), même à l'administrateur**, avec un message qui accuse à tort le rôle (\#534) ; il n'existe pas d'autre passage de la démonstration à la production sur la même installation — choix de sécurité existant : une installation ne se vide pas sans un geste de l'exploitant. La réinitialisation vide **toutes** les données comptables et commerciales (écritures, factures, avoirs, contacts, articles, taux, réglages…) et ramène la société à son état initial ; les **fichiers** déjà produits (PDF figés, pièces importées) restent sur le disque, orphelins (inventaire § 5) ; elle **conserve** les utilisateurs et les clés d'API — ceux créés pendant la démonstration passent en production : **revoyez-les** ; une clé d'API saine garde son **droit d'écriture** si elle l'avait (F5-3 de la P5) —, et la société garde son identité (la session reste ouverte) ; **sauf** sur une installation déjà touchée par le défaut \#528 (société effacée par une version antérieure) **qui compte aucune ou une société** (R4-5 de la P4 ; corrigé à la revue de code P1, A-1 = E-3 — à deux sociétés ou plus, la réinitialisation rend une erreur interne, AC 2.2, et la réparation est celle de la 15-7b3 au démarrage) : les utilisateurs qui désignaient la société effacée y sont rattachés, et les clés d'API qui la désignaient sont **révoquées**, puis rattachées à la société — elles paraissent, révoquées, sur la page des clés : à recréer si elles servent ; sur une installation touchée restée **sans société**, le choix de la langue fait la même réparation ; la session en cours doit être **rouverte** (reconnexion, ou attendre le renouvellement du jeton, au plus la durée de session plus une minute) — d'ici là, certains écrans de la société répondent par une erreur, d'autres s'affichent vides (F3-4 de la P3) ; la réparation au démarrage d'une installation touchée est écrite par la 15-7b3. Une installation qui porte **plusieurs sociétés provisoires** (défaut \#542 : un redémarrage avant la création de l'administrateur en ajoutait une) est réparée **au démarrage** (15-7b3, C-15-7-46). *(La cellule tient sur une ligne dans cette fiche ; au manuel, ce texte complète l'`\item \textbf{Parcours}` en plusieurs phrases, et les renvois aux passes entre parenthèses — « F3-4 de la P3 », « R4-5 de la P4 » — ne se recopient pas ; R3-6 de la P3.)* |
| `admin-manual.tex:1250-1259` (matrice du démarrage, R2-3/L-2 de la P2) | « vide & non → Crée la company stub seule » ; « vide & oui → Crée stub + admin » ; « existant & non → No-op » | « vide & non » : « Crée la company stub **si aucune société n'existe** (un redémarrage avant \keshpath{/setup} n'en ajoute plus, \#542), écran /setup actif » ; « vide & oui » : « Crée stub + admin, ou rattache l'admin à la société existante » (déjà inexact aujourd'hui, `bootstrap.rs:100-120`) ; la phrase sous le tableau sur la réparation de \#528 au démarrage est écrite par la 15-7b3 |
| `admin-manual.tex:967` (« Le bootstrap crée seulement une company provisoire ») | — | relire : « au boot » devient « au premier démarrage, sur une base sans société » (AC 13) |
| `user-manual.tex:2019-2023` | deux familles : la réinitialisation de la démonstration, et les gestes de session | **une** famille : les gestes de session |
| `user-manual.tex:2216` (glossaire, « Journal d'audit ») | « les deux familles d'opérations qui n'y figurent pas encore » | « la famille d'opérations qui n'y figure pas encore (les gestes de session) » |
| `admin-manual.tex:2244` (glossaire) | « voir les réserves de la section Conformité » | relire : ne doit pas contredire `:1821` et `:2004` |
| `user-manual.tex:2025-2038` (note) | « reste possible » | ajoute qu'elle laisse sa propre entrée, **première** du journal neuf |
| `user-manual.tex:177-189` (§ *Chemin A — Mode exploration*, réécrit par la 15-7b1 pour \#544 ; F3-7 de la P3, F4-2/R4-2 de la P4) | la démonstration et son renvoi à la réinitialisation, **avec sa condition** (report de F4-2 dans la 15-7b1, C-15-7-48) | une phrase : la réinitialisation (bouton « Réinitialiser pour la production » du bandeau de démonstration) n'est possible que si l'**administrateur de l'installation l'a autorisée au préalable** (variable `KESH_PRODUCTION_RESET`, voir le manuel d'administration) — sans cette autorisation, elle est refusée, **même à un administrateur** ; elle vide **toutes** les données de la société --- y compris ce que vous avez saisi pendant l'exploration --- et **conserve** les utilisateurs et les clés d'API |
| `user-manual.tex`, § *Consulter le journal d'audit* | entrées de configuration (15-7a2) et chargement de la démonstration en une entrée (15-7b1) | la remise à zéro explique le trou de numérotation qui la précède |

⚠️ **Greper la valeur, sans casse** (`grep -niE`) : `\b434\b`, `\b104\b`, `\b105\b`, `\b112\b`,
« efface encore », « deux familles » (minuscule à `user-manual.tex:2216`, majuscule à `:2019`),
`stub seule|company provisoire|placeholder` (AC 13, sites `:967` et `:1254`), `\b528\b`, `\b542\b`,
`\b534\b`, `Réinitialiser pour la production` (les renvois au bouton doivent tous dire sa condition,
C-15-7-48), `révoqu`,
`KESH.{1,2}PRODUCTION.{1,2}RESET` (le LaTeX écrit `KESH\_PRODUCTION\_RESET` : un `.` ne couvre pas
`\_`, F-4 de la P1 — le motif rend `admin-manual.tex:691`, `.env.example:279`,
`docker-compose.dev.yml:30`, et, la 15-11a mergée, `docker-compose.yml` et `docker-compose.prod.yml` — **une ligne par compose** : la 15-11a interdit de nommer la variable dans un commentaire des compose ; **plus, au manuel et au CHANGELOG, les mentions que la 15-11a ajoute** (AC12 f et AC13 : le `lstlisting` des gestes de mise à jour, où `docker compose config` doit montrer `KESH_PRODUCTION_RESET`, la liste « relisez » de l'avertissement avant `up -d`, le contrôle de la mise à jour, et l'entrée du CHANGELOG) — **à relever au rebase par le texte, sans les compter d'avance** : elles ne sont pas des résidus à corriger, la 15-11a les écrit exactes — et `admin-manual.tex:691` reste la seule cellule de tableau), `séquence d.installation` (apostrophe typographique dans le PDF : un grep à
apostrophe droite rend un faux négatif), sur `docs/manual/fr/*.tex`, le PDF aplati, `README.md`,
`website/`. La ligne v0.12.1 du README est historique.

**12. CHANGELOG** : sous `## [0.13.0] — Non publié`, `### Corrigé` : la remise à zéro s'inscrit au
journal d'audit et y laisse sa propre entrée (#434) ; elle vide toutes les données de la société (#279)
et ne laisse plus l'utilisateur rattaché à une société effacée (#528) ; sur une installation déjà
touchée, la remise à zéro et — si elle ne compte aucune société — le choix de la langue rattachent les
utilisateurs à la société et **révoquent** les clés d'API qui désignaient la société effacée (rattachées
à la société, elles paraissent révoquées sur la page des clés) — la session ouverte doit être rouverte
(la réparation au démarrage et à la restauration est la ligne de la 15-7b3) ; ses trois gardes sont
évaluées sous le verrou de l'effacement. Et : un redémarrage avant la création de l'administrateur
n'ajoute plus de société provisoire (#542 ; les sociétés déjà ajoutées sont supprimées au démarrage,
ligne de la 15-7b3). `### Documentation` (ou sous `### Corrigé` si la section n'existe pas) : le manuel
dit que sortir de la démonstration exige `KESH_PRODUCTION_RESET`, posé par l'exploitant le temps de la
réinitialisation (#534 pour le message de refus).

**13. Le démarrage sans administrateur n'insère une société provisoire que sur une base sans société**
(#542, R1/F-1 de la P1, C-15-7-34). Dans `ensure_admin_user` (`auth/bootstrap.rs`), la branche
`(0, false)` — aucun utilisateur, aucune variable d'administrateur (`:72-81`) — n'appelle
`insert_stub_company` que si **`company_count == 0`**, compteur déjà lu en tête de la fonction
(`:52-55`, « lecture unique … partagée par toutes les branches ») ; sinon elle journalise (`info`) et
n'insère rien. Elle rend `Ok(0)` dans les deux cas. Le cas 2 (`(0, true)`, `:83`, garde `:100`) se garde déjà ainsi :
la correction aligne le cas 1 sur lui. Comportement des autres branches inchangé. Sans cette garde,
chaque redémarrage avant `/setup` ajoutait une société, et `seed_demo` comme la remise à zéro
rendaient 500 `Invariant` ; les installations **déjà** touchées sont réparées **au démarrage** par
la **15-7b3** (F4-3 de la P4, C-15-7-46 : la remise à zéro, qui les réparait dans la rédaction
précédente, n'est atteignable par aucun écran sur cet état). D'où `refs #542` ici, `closes #542` à la
15-7b3.

**14. → 15-7b3.** La réparation au démarrage d'une installation déjà touchée par #528 (AC ajouté à la P2,
C-15-7-38) est sortie à la P3 avec son test 15, ses trois mutations, son action d'audit et ses lignes
de manuel et de CHANGELOG (C-15-7-40) ; la 15-7b3 l'étend à la restauration d'une sauvegarde
(C-15-7-44), y appelle la règle des principaux orphelins posée ici (AC 4, C-15-7-45), et y ajoute, à la
P4 de cette fiche, la réparation au démarrage des installations touchées par #542 (C-15-7-46).
Numéro conservé pour la traçabilité des passes.

## Tasks / Subtasks

- [x] **T1 — `kesh-db`** (AC 2, 3, 4) — `companies::{reset_to_stub_in_tx, insert_stub, reattach_orphan_principals_in_tx}` (et `OrphanPrincipals`, `RevokedApiKey` ; doc-comment : la règle, révoquer puis repointer, ses quatre appelants), constantes stub déplacées ; `onboarding::reset_state_in_tx` ; `backup::{RESET_PRESERVED_TABLES, reset_cleared_tables}` et leurs deux tests (partition, schéma, trois règles) ; le littéral `"(en cours de configuration)"` de `test_fixtures.rs:477-480` remplacé par la constante descendue (F-7 de la P1, DRY).
- [x] **T2 — `kesh-seed::reset_demo`** (AC 2–7) — `serde_json` est ajouté à `crates/kesh-seed/Cargo.toml` par la 15-7b1 (T1) pour `installation.demo_seeded` : le vérifier, l'ajouter sinon (F-6 de la P1) ; `SeedError::ResetForbidden` (`StepAlreadyCompleted`, posée par la 15-7b1, sert la garde d'étape), `ResetOutcome` à champs publics, `SeedAttemptError` (posé par la 15-7b1) étendu de `ResetForbidden`, `is_seed_retryable` réemployé ; `retry_with`, transaction unique par essai, `close_on_drop()` après chaque `acquire`, toute erreur sqlx par `map_db_error`, rollback dont l'erreur ne masque jamais l'erreur d'origine, gardes, base sans société, plus d'une société ⇒ `Invariant`, vidage dérivé, stub en place, principaux orphelins par `reattach_orphan_principals_in_tx`, entrée, FK rétablis avant commit.
- [x] **T3 — Handler et bootstrap** (AC 2, 4, 7, 13) — `reset` : `Extension(current_user)`, correspondance `SeedError` → `AppError` de l'AC 2, plus de transaction ni de garde (`get_or_init_state` et relecture conservés) ; branche « aucune société » d'`ensure_company_with_language` sur `insert_stub` + `reattach_orphan_principals_in_tx` (détails de `company.created`, AC 4) ; `insert_stub_company` du bootstrap sur `insert_stub` ; branche `(0, false)` de `ensure_admin_user` **gardée par `company_count == 0`** (#542) ; commentaire `frontend/src/routes/onboarding/+layout.svelte:17-19` (AC 10).
- [x] **T4 — Libellés et registre** (AC 8, 9 — une action).
- [x] **T5 — Doc-comments, Pattern 5 et liste d'exceptions** (AC 10).
- [x] **T6 — Tests** (Dev Notes § Tests), dont le test de composant frontend, le test 13 (prédicat de rejeu), le test 14 (bootstrap, `mod tests` de `auth/bootstrap.rs`), le test 10b (`mod tests` de `kesh-db/src/backup.rs`) ; et **l'assertion « id mort » du test 7 de la 15-7a2** (`crates/kesh-api/tests/onboarding_audit_e2e.rs`) remplacée par le rattachement effectif (test 6c, R4 de la P1) ; le test 6f (fonction seule) dans `crates/kesh-db/tests/companies_repository.rs`.
- [x] **T7 — Manuels, PDF, CHANGELOG** (AC 11, 12) — ⛔ toute recette de sortie de la démonstration écrite aux manuels (`:1314`, `:691`) prescrit `docker compose up -d` après chaque changement de `KESH_PRODUCTION_RESET` (pose, puis retrait), jamais « redémarrer » ni `restart`, qui ne relit pas `.env` (15-11a, AC12 f, C83, C84) ; `user-manual.tex:177-189` ne prescrit aucune manipulation d'exploitant : il renvoie au manuel d'administration, rien à y dire du redémarrage — dont `admin-manual.tex:1314` (l'`\item \textbf{Parcours}`, complété en plusieurs phrases ; la cellule de la fiche tient sur une ligne, ses renvois aux passes ne se recopient pas — R3-6), la matrice du démarrage `:1250-1259`, `:967`, `user-manual.tex:177-189` (F3-7, F4-2) et, après rebase, le seul contrôle au PDF aplati que la cellule `KESH\_PRODUCTION\_RESET` (`:691`) se lit entière — la mise en page des dix tableaux de `sec:env-vars` est faite par la **15-11a** (AC12 (j), C-15-7-54) ; les numéros de ligne des `.tex` se relocalisent par le texte après rebase (F5-2) ; **le renvoi au bouton de la 15-7b1** (`user-manual.tex:179-189`) dit sa condition (C-15-7-48) — si la 15-7b1 est livrée sans, le corriger ici.
- [x] **T8 — Gates** : **avant le merge**, la 15-11a est mergée (C-15-7-51) : `grep -n "PRODUCTION_RESET" docker-compose.yml docker-compose.prod.yml` rend une ligne par fichier (la 15-11a interdit de nommer la variable dans un commentaire des compose ; si le grep en rend davantage, c'est un défaut de la 15-11a à lui renvoyer, non à tolérer ici) — ⚠️ le contrôle de l'AC 11 sur `docs/manual/fr/*.tex` et `CHANGELOG.md`, lui, rend **plus** de lignes qu'avant la 15-11a : ses mentions y sont attendues, listées à l'AC 11, et `KESH_PRODUCTION_RESET=true docker compose -f docker-compose.yml config | grep PRODUCTION_RESET` montre la variable transmise — sorties collées au Dev Agent Record ; sinon la PR attend ; `kesh-db` touché ⇒ gate complet même en cours de boucle ; base remise à zéro avant ; frontend : `npm run check`, `test:unit` ; E2E complet au dernier commit de code. ⚠️ `onboarding.spec.ts:57` (« bannière démo — reset redirige vers onboarding ») est un **échec attendu** (KF-029, #97, `docs/testing.md:351`) : il rougit avant **et** après la story et **ne vaut pas preuve** ; les preuves sont les tests 5 et 12. Cause probable, à signaler sur #97 : le montage E2E ne pose pas `KESH_PRODUCTION_RESET` (absent de `docs/testing.md` et de `frontend/playwright.config.ts`), si bien que la remise à zéro d'une démo à l'étape 3 y est refusée en 403.

## Dev Notes

### Points de vigilance

- **`reset_demo` et la connexion dédiée** : `SET FOREIGN_KEY_CHECKS` est une variable de session ; la
  transaction s'ouvre **sur cette connexion** (`sqlx::Connection::begin(&mut *conn)`), et
  `insert_in_tx` prend `&mut Transaction<'_, MySql>` — compatible. La closure de `retry_with` est
  `Fn` : elle acquiert sa connexion à chaque essai, la marque `close_on_drop()`, plutôt que d'emprunter
  une connexion extérieure. Pas de `TRUNCATE` (DDL, commit implicite) : des `DELETE`.
- **Lecture verrouillante d'`audit_log`** : sous REPEATABLE READ, un `SELECT … FOR UPDATE` sans
  `WHERE` **devrait** poser des verrous de clé suivante jusqu'au supremum — une insertion concurrente
  attendrait le commit. **Conjecture, non vérifiée** contre MariaDB pour la forme agrégée (`COUNT(*),
  MIN(id), MAX(id)`) (F3-6 de la P3) : la plage relevée est voulue égale à celle qui est effacée, et la
  garde `rows_affected` de l'étape 4 rend `Invariant` si elle ne l'est pas.
- **Concurrence avec les routes métier** : les routes qui suivent le Pattern 5 prennent `companies` et
  se sérialisent derrière la remise à zéro, puis voient l'état d'après (leurs lignes ont disparu :
  `NotFound`, ou recréation sur une société neuve). Celles qui ne prennent pas `companies` —
  `invoices::validate_invoice` (`invoices → fiscal_years → invoice_number_sequences →
  journal_entries`), un écrivain d'audit qui tient déjà une ligne métier,
  `users::update_role_and_active_in_tx` — verrouillent dans l'ordre inverse de certains verrous de la
  remise à zéro : **un interblocage est possible**. InnoDB désigne une victime (1213) : si c'est la
  remise à zéro, l'essai est annulé en entier (rien d'effacé) et `retry_with` le rejoue (C-15-7-22) ; si
  c'est l'autre transaction, elle rend son erreur ordinaire. Limite à écrire au Dev Agent Record :
  l'interblocage **de `reset_demo` lui-même** n'est pas provoqué par un test — il n'est pas
  reproductible de façon déterministe dans son corps ; ce qui l'est est couvert : la conversion de
  toute erreur sqlx par `map_db_error` (imposée par le type `SeedAttemptError`, AC 2), le prédicat sur
  un vrai 1213 (test 13), et `retry_with` (`kesh-db/src/retry.rs`).
- **`seed_demo` après une remise à zéro** : la société stub existe (AC 4) ; le premier bloc
  (`len() != 1`) passe. Aucun changement.
- **`seed_demo` concurrent d'une remise à zéro** : la préservation de l'identité de la société élargit
  une fenêtre — avant, `reset` supprimait `companies` et un `seed_demo` concurrent échouait sur
  `NotFound` ou une clé étrangère sans rien laisser ; après, la société survit (même id, `version + 1`),
  et une remise à zéro entre `companies::update` et `bulk_create_from_chart`/`create_for_seed` laisse
  comptes et exercice sur la société d'une installation revenue à l'étape 0 ; la dernière transaction
  rend `StepAlreadyCompleted`, et un `start-production` ultérieur trouve un plan et un exercice déjà là.
  Régression induite par #528, mono-utilisateur, étroite : **suivie par #538** (« `seed_demo` et la
  remise à zéro se sérialisent »), non corrigée ici.
- **Bootstrap, cas 1 — la cause de #542, fermée ici** (AC 13 ; L-7 de la P4 l'avait classé hors
  périmètre, R1/F-1 de la P1 et la décision de l'orchestrateur l'y ramènent) : sans utilisateur ni
  variable d'admin, `auth/bootstrap.rs:72-75` insérait un stub **à chaque démarrage**, sans regarder
  `company_count`. Plusieurs redémarrages avant `/setup` produisaient plusieurs sociétés, et `seed_demo`
  comme la remise à zéro rendaient alors 500 `Invariant`. La garde `company_count == 0` ferme la cause.
  **La réparation des installations déjà touchées passe à la 15-7b3, au démarrage** (F4-3 de la P4,
  C-15-7-46) : la rédaction précédente la confiait à la remise à zéro, que cet état n'atteint par aucun
  écran (le bouton vit dans `DemoBanner`, rendu seulement si `isDemo`, `(app)/+layout.svelte:336-337` ;
  `seed_demo` y rend 500, l'installation ne devient donc jamais une démonstration). Sur cet état, la
  remise à zéro de cette fiche rend 500 `Invariant` (test 6d (ii)) — constaté, inatteignable par écran.
  Les sociétés superflues sont inoffensives pour le parcours de production **parce que** tous les sites
  mono-société lisent `MIN(id)` (`setup.rs:130`, `routes/onboarding.rs:90`, `:858`, `:913`, `:918`,
  `admin.rs:279`, `:376`, `backup.rs:283`).
- **Bootstrap, réparation de #528 et de #542** : en **15-7b3** (C-15-7-40, C-15-7-46) ; cette fiche ne
  touche du bootstrap que les constantes, `insert_stub` et la garde de #542.
- **Tests existants qui changent de sens** : `onboarding_e2e.rs:395-560` (réinitialisations : la
  garde quitte le handler mais les codes restent ; `onboarding_e2e.rs:476-525` crée une société, donc
  l'invariant tient ; vérifier les états relus) ; tout test qui attendrait `companies` vide ou un
  nouvel `id` après une remise à zéro (grep `onboarding/reset` et `reset_demo` dans `crates/*/tests`) ;
  `frontend/src/lib/features/onboarding/onboarding.svelte.test.ts:95` (mock de `resetDemo` à
  `isStub: false` : reste valide, le test 12 ajoute le cas `true`) ; **le test 7 de la 15-7a2**
  (`crates/kesh-api/tests/onboarding_audit_e2e.rs`), dont l'assertion « id mort » est remplacée par le
  rattachement effectif (test 6c) ; les tests du bootstrap (`auth/bootstrap.rs`, `mod tests`) :
  `bootstrap_db_empty_no_env_creates_stub_only` (`:397`) reste vert (un seul démarrage).
- **Le dialogue de confirmation sous-estime ce qui est effacé** (hors périmètre) :
  `demo-reset-confirm-body` (« Toutes les données de démonstration seront supprimées »,
  `crates/kesh-i18n/locales/fr-CH/messages.ftl:67`, quatre locales) ne dit ni la piste d'audit, ni ce
  qui a été saisi pendant la démonstration, ni que les utilisateurs et les clés sont conservés :
  **#534** (`refs #534` ; L4-2 de la P4 — la CR existait déjà). Même issue : `DemoBanner.svelte:30-37`
  rend **tout** 403 par « Seul un administrateur peut réinitialiser cette instance »
  (`demo-reset-forbidden`) — or le 403 `ONBOARDING_RESET_FORBIDDEN` que l'AC 2 conserve (démonstration
  à l'étape ≥ 3 sans `KESH_PRODUCTION_RESET`, test 7) frappe aussi un administrateur : le message y
  ment. Défaut antérieur, non corrigé ici.
- **Sortir de la démonstration exige un geste de l'exploitant** (F4-2/R4-2 de la P4, C-15-7-48) : une
  démonstration est **toujours** à l'étape 3 (`seed_demo`, `update_step(pool, 3, true, …)`), la garde
  `step_completed > 2 && !KESH_PRODUCTION_RESET` (`routes/onboarding.rs:276`) refuse donc **toute**
  réinitialisation d'une démonstration sur une installation par défaut (drapeau vide,
  `docker-compose.dev.yml:30`, `.env.example:279`), et toutes les routes du parcours de production
  refusent `is_demo` (`routes/onboarding.rs:305`, `:331`, `:366`, `:454`) : sans le drapeau, la
  démonstration est une **impasse**. C'est un **choix de sécurité existant** (une installation ne se vide
  pas sans geste de l'exploitant), que cette fiche **conserve** et **écrit** — manuels (AC 11, lignes
  `:691`, `:1314`, `user-manual.tex:177-189`) et renvoi de la 15-7b1 ; l'orchestrateur le signale sur
  #534 (le libellé) — la question produit d'une sortie sans geste d'exploitation n'est pas tranchée ici.
  ⚠️ **Le geste n'atteint le processus qu'une fois la 15-11a mergée** (F5-1 de la P5, C-15-7-51) : les
  compose de production (`docker-compose.yml`, `docker-compose.prod.yml`) listent leurs variables
  explicitement, sans `env_file`, et omettent `KESH_PRODUCTION_RESET` (comme une vingtaine d'autres, dont
  le SMTP — #550). La 15-11a les transmet toutes et pose un test contre la dérive ; cette fiche ne touche
  pas aux compose, et ne se merge qu'après elle (en-tête, T8).
- **Statut de la fiche** : `ready-for-dev` pendant les passes de validation est la convention de
  l'epic ; la clôture de la validation se lit au Change Log, pas au statut.

### Tests

Fichier `crates/kesh-api/tests/onboarding_audit_e2e.rs` (créé par la 15-7a2), même patron : lecture
d'`audit_log` par les helpers de `crates/kesh-api/tests/common/mod.rs`, dont `audit_sequence` ajouté
par la 15-7a2 (séquence exacte, contenu des détails), DDL de test par **`sqlx::raw_sql`**.

| # | Test | AC |
|---|---|---|
| 4 | Remise à zéro réussie (`KESH_PRODUCTION_RESET`, patron `onboarding_e2e.rs:476-525`), après `seed-demo` **plus** une facture validée, un avoir, un contact, un article, **une clé d'API** de la société (saine : elle désigne la société conservée) et **un jeton de réinitialisation de mot de passe** créés au montage (L4-3 de la P4 : sans eux, « inchangés » était vrai à vide). **Relevé avant** la remise à zéro (R3 de la P1) : `COUNT(*)` de **chaque** table de `reset_cleared_tables()` ; (a) les tables que le montage peuple, **nommées** dans le test — au moins `accounts`, `fiscal_years`, `vat_rates`, `company_invoice_settings`, `contacts`, `products`, `invoices`, `invoice_lines`, `credit_notes`, `credit_note_lines`, `journal_entries`, `journal_entry_lines`, `audit_log` — sont à **`> 0` avant** (sinon le montage est à compléter, non la liste) ; (b) les tables à **0 avant** forment une **liste fermée écrite** dans le test (`bank_profiles`, `reconciliation_rules`, `payment_batches`, `supplier_invoices`… — établie au développement depuis le relevé, et assertée **égale** à l'ensemble relevé : une table qui se met à être peuplée, ou qui cesse de l'être, rougit) — ce qui n'est pas amorcé est dit, non ignoré. **Après** : exactement une ligne d'audit, `installation.reset`, `audit_entries_erased` = compte d'avant, `erased_id_min`/`max` exacts, `id > erased_id_max`, `company_recreated = false`, `tables_cleared` = `reset_cleared_tables()`, `api_keys_revoked = []`, `api_keys_repointed = 0` ; **chaque** table de `reset_cleared_tables()` sauf `audit_log` à `COUNT(*) = 0` (#279, la classe entière) ; `users`, `api_keys` (la clé saine **non révoquée**, `version` inchangée), `refresh_tokens`, `password_reset_tokens` inchangés | 2, 3, 5 |
| 5 | **#528 et colonnes du stub** (R4-1/F-1, F-4 de la P4). Montage : après `seed-demo`, `UPDATE companies` porte chacune des **19** colonnes non exclues à une valeur **différente** de celle du stub et conforme aux `CHECK` (`name = 'Démo perturbée'`, `address = 'x'`, `ide_number = 'CHE109322551'`, `org_type = 'Pme'`, `accounting_language = 'DE'`, `instance_language = 'DE'`, `country = 'FR'`, `is_stub = FALSE`, les quatre `address_*` = `'x'`, `address_country = 'FR'`, `first_name`, `last_name`, `email`, `phone`, `website` non nuls, `books_locked_through = '2024-12-31'`) — liste **fermée** `PERTURBED` du test. `company_id` relevé avant (`GET /api/v1/companies/current`) ; la réponse de `reset` porte **`isStub == true`** ; après la remise à zéro, **avec le même jeton** : `GET /api/v1/companies/current` → **200**, même `company.id` ; `users.company_id` inchangé. **Inventaire** : l'ensemble des colonnes de `information_schema.COLUMNS` (`TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'companies'`) égale `PERTURBED` ∪ {`id`, `version`, `created_at`, `updated_at`} — une colonne neuve rougit le test tant qu'elle n'y est pas classée. **Référence** : construite **après** la remise à zéro et **avant** l'appel `language`, dans une transaction **annulée** — `begin` → `companies::insert_stub(&mut *tx, Language::Fr)` → `SELECT` des deux lignes → `rollback` —, de sorte que la base ne compte jamais deux sociétés au moment d'une remise à zéro (sinon AC 2.2 : 500, cf. test 6d). **Comparaison** : pour **chaque** colonne lue dans `information_schema.COLUMNS` **sauf** la liste fermée `id`, `version`, `created_at`, `updated_at`, la valeur de la société remise à zéro égale celle de la référence (`NULL` compris : comparaison par `<=>` en SQL). Puis `language` (`de`) → `company.updated {instance_language: fr → de}`, **pas** de `company.created` | 4 |
| 6a | Installation atteinte par #528 **avec** société (montage : une clé d'API **active**, créée par `generate_pat` + `api_keys::create_in_tx`, jeton gardé ; puis `users.company_id` et `api_keys.company_id` pointant un id inexistant, sous `FOREIGN_KEY_CHECKS=0` sur une connexion **détachée du pool**, `pool.acquire().await?.detach()`) ⇒ après remise à zéro, `users.company_id` = la société conservée, `details.users_repointed = 1` ; la clé est **révoquée** (`revoked_at` non nul) **et** désigne la société ; `find_active_auth_by_key_hash` de son condensat rend `None` ; `details.api_keys_revoked` = `[{id, name, created_by_user_id, created_at, last_used_at}]` de la clé, `api_keys_repointed = 1` | 4, 5 |
| 6b | Installation atteinte par #528 **sans** société (montage : aucune ligne `companies`, `users.company_id` mort, étape 0) ⇒ la remise à zéro répond **200** (et non 500) ; une société stub existe, `details.company_recreated = true`, `details.company_id` = son id, `users.company_id` = ce même id, `users_repointed = 1`, la clé active orpheline du montage (comme 6a) révoquée et repointée, `audit_log.company_id` de l'entrée = ce même id ; puis **nouvelle connexion** (login) ⇒ `GET /api/v1/companies/current` → 200 avec ce même id — l'ancien jeton n'est pas asserté (limite de l'AC 4) | 2, 4, 5 |
| 6c | Même montage que 6b (clé active orpheline comprise), mais **`language`** au lieu de la remise à zéro — **le chemin de F4-1/R4-1 de la P4** : choisir la langue ne doit réveiller aucune clé ⇒ `users.company_id` = la société créée ; la clé est **révoquée** et désigne la société créée, `find_active_auth_by_key_hash` rend `None` ; `company.created` porte `users_repointed = 1`, `api_keys_revoked` (la clé), `api_keys_repointed = 1` ; son `audit_log.company_id` = `entity_id` — remplace l'assertion « id mort » du test 7 de la 15-7a2 ; puis nouvelle connexion ⇒ `GET /api/v1/companies/current` → 200 avec l'id créé | 4 |
| 6d | Deux sociétés au montage — (i) la seconde **non provisoire** (`companies::create`) ; (ii) **#542** : la seconde **provisoire**, sans utilisateur ni clé (`companies::insert_stub`), état que la remise à zéro ne répare plus (C-15-7-46) ⇒ chaque fois remise à zéro **500** (`Invariant`), toutes les tables, les sociétés et la piste intactes | 2 |
| 6e | **→ 15-7b3, test 4** (F4-3 de la P4, C-15-7-46) : la suppression des sociétés provisoires superflues est faite au démarrage. Numéro conservé | — |
| 6f | **Fonction seule** — `reattach_orphan_principals_in_tx`, dans `crates/kesh-db/tests/companies_repository.rs`, `#[sqlx::test(migrations = "./test-schema")]` ; montage sur connexion détachée du pool (`FOREIGN_KEY_CHECKS=0`) : un utilisateur orphelin, une clé **active orpheline**, une clé **déjà révoquée** orpheline, une clé **saine** (de la société). (i) `Some(société)` ⇒ `user_ids` = l'utilisateur, rattaché ; la clé active orpheline révoquée (`version` + 1) et repointée ; la clé déjà révoquée repointée, `revoked_at` et `version` **inchangés** ; la clé saine **intacte** ; `api_keys_revoked` = la seule clé active orpheline, nom et dates exacts ; `api_keys_repointed = 2`. (ii) `None` ⇒ la clé active orpheline révoquée et **non** repointée, l'utilisateur non rattaché, `user_ids` vide, `api_keys_repointed = 0`. (iii) aucun orphelin ⇒ `is_empty()`, rien d'écrit | 4 |
| 7 | Gardes, par HTTP (elles n'existent plus qu'en `reset_demo`) : étape 7 ⇒ 400 ; `is_demo = false`, étape 3 ⇒ 403 ; `is_demo = true`, étape 3, drapeau absent ⇒ 403 ; chaque fois toutes les tables et la piste **intactes**. Séquentiel : il prouve les gardes, **pas** qu'elles sont évaluées sous le verrou de l'effacement — c'est le 7b | 2 |
| 7b | **Gardes sous le verrou, de façon déterministe** (F-2 de la P1) — démonstration à l'étape 3, `is_demo = true`, **peuplée** (F3-5 de la P3 : une table à 0 avant et après ne prouve rien) : montage par `seed-demo` plus une écriture validée ; **relevé avant** du `COUNT(*)` de chaque table de `TABLES_TO_TRUNCATE`, avec `accounts`, `fiscal_years`, `journal_entries`, `journal_entry_lines` et `audit_log` assertées **`> 0`** (patron du test 4). Une connexion **A**, dédiée hors du pool (patron du test 7 de la 15-7a1), ouvre une transaction et exécute `kesh_db::repositories::onboarding::LOCK_SQL` (verrou sur `onboarding_state`) ; `kesh_seed::reset_demo(&pool, actor, true)` est lancé dans une tâche (`tokio::spawn`) ; le test attend qu'il soit **bloqué** par **`kesh_db::test_fixtures::attendre_une_requete_en_cours(&pool, &["onboarding_state", "FOR UPDATE"], || handle.is_finished())`** (`test_fixtures.rs:560`, Story 25-3-a-1 ; R2-1/F-2 de la P2 : `information_schema.INNODB_TRX` exige le privilège `PROCESS`, que l'utilisateur `kesh` du gate local n'a pas, et n'est pas filtrable par base — sous nextest à huit fils, l'attente d'un test voisin aurait satisfait la boucle ; l'aide lit `PROCESSLIST` filtrée sur `DB = DATABASE()`, sans privilège, délai de 10 s intégré ; ses motifs sont couplés à la forme de `LOCK_SQL`) ; elle doit rendre **`true`** (asserté) ; A passe `step_completed = 7` et **commite** ⇒ `reset_demo` rend **`SeedError::StepAlreadyCompleted`** ; toutes les tables et la piste sont **identiques au relevé d'avant** | 2 |
| 7c | Garde de l'étape 1 (R6 de la P1) : `onboarding::delete_state` au montage, puis `kesh_seed::reset_demo` **appelé directement** (le handler recréerait la ligne par `get_or_init_state`) ⇒ `SeedError::Db(DbError::Invariant(_))`, rien d'effacé | 2 |
| 8 | **Échec injecté et connexion jamais rendue** (F-2 de la P4) : sur un pool à **une seule connexion** construit dans le test (`MySqlPoolOptions::new().max_connections(1)`, options du pool `#[sqlx::test]`), relever `SELECT CONNECTION_ID()` par `pool1` (connexion relâchée aussitôt) ; déclencheur `BEFORE INSERT ON audit_log` — **le premier du dépôt** (`grep -rln "CREATE TRIGGER" crates` est vide, L4-4 de la P4) : créé par `sqlx::raw_sql`, corps `SIGNAL SQLSTATE '45000'` ; pré-requis à vérifier au montage — journal binaire inactif (`SELECT @@log_bin`), faute de quoi `CREATE TRIGGER` exige `SUPER` ou `log_bin_trust_function_creators` (le privilège `TRIGGER` est couvert : `scripts/mariadb-init/01-dev-grants.sql:38`, et la CI se connecte en `root`) : `assert_eq!(log_bin, 0, …)` avec un message qui **nomme le pré-requis**, jamais de sortie anticipée ni de `SET GLOBAL` (R5-5 de la P5 : un `return` ferait un test muet ; ni `docker-compose.dev.yml` ni `ci.yml` n'activent `--log-bin`, le cas est théorique) ; un échec de montage ne se lit pas comme un échec du test ⇒ `kesh_seed::reset_demo(pool1, …, true)` rend une erreur ; `COUNT(*)` de chaque table de `TABLES_TO_TRUNCATE` identique à avant, société et `onboarding_state` inchangés ; **puis, par `pool1`** : `CONNECTION_ID()` **différent** de celui relevé (la connexion de l'essai a été fermée, non rendue) — **c'est la preuve** — et `@@SESSION.foreign_key_checks = 1`, simple ceinture : sur une connexion neuve il vaut 1 par construction (F5-4 de la P5) ; puis `DROP TRIGGER`, `COUNT(*)` d'`audit_log` relevé, remise à zéro rejouée avec succès, et le **`ResetOutcome` rendu asserté** (R4-4) : `company_id` = l'id de la société du montage, `company_recreated == false`, `audit_entries_erased` = le compte relevé, `principals.is_empty()` | 2, 6 |
| 9 | Piste vide (montage : `DELETE FROM audit_log` à l'étape 0) ⇒ `audit_entries_erased = 0`, `erased_id_min`/`max` à `null` | 5 |
| 10 | Unitaire : partition `RESET_PRESERVED_TABLES` ∪ `reset_cleared_tables()` = `TABLES_TO_TRUNCATE`, ordre conservé, six noms préservés présents. ⚠️ **Vrai en partie par construction** (`reset_cleared_tables()` est dérivé) : seule l'appartenance des six noms mord (renommage d'une table) — ce test **n'est pas** la garde de la partition, c'est le 10b | 3 |
| 10b | **Schéma** — dans `mod tests` de `crates/kesh-db/src/backup.rs`, à côté de `backup_inventory_matches_schema` (`:626`), attribut `#[sqlx::test(migrations = "./test-schema")]` (graphie exigée par `test_schema_guard.rs:62`, F-7 de la P1) ; `information_schema.KEY_COLUMN_USAGE` : les **trois** règles de l'AC 3, la liste fermée d'exceptions (`audit_log`), et une **règle 4** (R2-5 de la P2) : les tables de `RESET_PRESERVED_TABLES` qui portent une clé vers `companies` sont **exactement** {`users`, `api_keys`} — c'est le fait sur lequel repose la règle des principaux orphelins (AC 4 : seuls `users` et `api_keys` sont à rattacher) et le prédicat de « superflue » de la 15-7b3 ; une future table préservée portant un `company_id` rougit le test | 2, 3, 4 |
| 12 | **Frontend** (Vitest) : dans `onboarding.svelte.test.ts`, `resetDemo` résolu avec `isStub: true` ⇒ `onboardingState.isStub === true` ; test de composant `frontend/src/routes/onboarding/onboarding-layout.test.ts` (`@testing-library/svelte`, patron `src/routes/(app)/homepage-page.test.ts`) : `onboardingState.isStub` est un état **de module en lecture seule** (`onboarding.svelte.ts:34-35`), non injectable par une prop — le test **mocke le module d'API d'onboarding** (`vi.mock`, réponse portant `isStub` vrai puis faux) et **appelle `onboardingState.resetDemo()`** (ou `fetchState()`) **avant** `render`. Dépendances réelles de `+layout.svelte` (R4-5, L-4 de la P4) : il n'importe que `onboardingState` et `i18nMsg` (`$lib/features/onboarding/onboarding.svelte`, `:2`) et `appVersion` (`$lib/shared/utils/app-version.svelte`, `:3`) — mocker `appVersion`, **rien** de `$app/*` ni d'état d'authentification ; et il rend `{@render children()}` (`let { children } = $props()`, `:5`, `:31`) : passer un snippet `children` construit par `createRawSnippet` (`svelte`), faute de quoi le rendu lève une erreur. `isStub` vrai ⇒ `[data-testid="onboarding-stub-notice"]` présent, faux ⇒ absent | 4 |
| 13 | **Prédicat de rejeu sur un vrai 1213** (le prédicat, pas l'interblocage de `reset_demo`) : une table de test créée par `sqlx::raw_sql` (`CREATE TABLE reset_retry_probe (id INT PRIMARY KEY) ENGINE=InnoDB`, deux lignes 1 et 2 ; R4-6 de la P4 — `onboarding_state` est un singleton) ; deux **connexions dédiées hors du pool** ouvrent chacune une transaction, A verrouille la ligne 1, B la ligne 2 (`SELECT … FOR UPDATE`), puis A demande la 2 et B la 1 **concurremment** (`tokio::join!` des deux requêtes, ou `tokio::spawn` — sans concurrence le cycle ne se forme pas) ; le détecteur d'InnoDB désigne une victime, dont l'erreur est **1213** ; elle passe par `map_db_error` dans `SeedAttemptError::Db` ⇒ `is_seed_retryable` **vrai** (R3-1 de la P3). Une attente expirée (1205, `SET SESSION innodb_lock_wait_timeout = 1`, patron du test 7 de la 15-7a1) ⇒ **faux** ; `StepAlreadyCompleted`, `ResetForbidden`, `Db(DbError::NotFound)` ⇒ **faux**. Le prédicat est `is_seed_retryable` sur `SeedAttemptError`, partagé avec la dernière transaction de `seed_demo` (15-7b1) : ce test le prouve pour les deux flux | 2 |
| 14 | **Bootstrap, #542** — dans `mod tests` de `crates/kesh-api/src/auth/bootstrap.rs`, à côté de `bootstrap_idempotent_on_empty_db` (`:456`), attribut `"../kesh-db/test-schema"` : `test_config_no_env()`, **deux** appels d'`ensure_admin_user` sur base vide ⇒ `COUNT(*) FROM companies = 1`, chaque appel rend `Ok(0)`, `COUNT(*) FROM users = 0` (« deux démarrages sans utilisateur → une société », attendu de #542) ; puis un troisième appel ⇒ toujours 1 | 13 |

⚠️ **Prouver que les tests mordent** — chaque mutation seule, puis le fichier **touché** : remettre une
liste de tables en dur à **huit** entrées (test 4 rouge) ; retirer le `reset_to_stub_in_tx` au profit
d'un `DELETE FROM companies` (test 5 rouge) ; retirer `website = NULL` de `reset_to_stub_in_tx` (test 5
rouge : la valeur perturbée survit) ; retirer `website` de `PERTURBED` (test 5 rouge : inventaire) ;
retirer l'appel à `reattach_orphan_principals_in_tx` de la branche « aucune société » (test 6c rouge) ;
rendre `Invariant` sur zéro société (test 6b rouge) ; accepter plus d'une société en gardant la
première (test 6d rouge : 200 au lieu de 500) ; **dans `reattach_orphan_principals_in_tx`** (C-15-7-45) :
repointer les clés sans les révoquer — la règle d'avant la P4 (tests 6c et 6f (i) rouges : clé active,
jeton accepté) ; révoquer sans repointer quand `target` est donné (test 6f (i) rouge : `company_id` mort) ;
retirer le filtre `revoked_at IS NULL` de l'étape 2 (test 6f (i) rouge : `revoke_in_tx` rend
`OptimisticLockConflict` sur la clé déjà révoquée) ; retirer le prédicat d'orphelin de la sélection des
clés — révoquer toutes les clés actives (tests 4 et 6f (i) rouges : clé saine révoquée) ; ne pas révoquer quand `target = None`
(test 6f (ii) rouge) ;
lire l'état par `onboarding::get_state` (lecture non verrouillante) au lieu de `lock_state_in_tx`
dans `reset_demo` — le `FOR UPDATE` retiré (test 7b rouge : la garde lit l'étape 3 sans attendre A ;
`reset_demo` ne se bloque qu'à l'étape 6, sur l'`UPDATE` d'`onboarding_state`, **après** avoir vidé
les tables, puis réussit quand A commite — tables effacées au lieu de `StepAlreadyCompleted` ; sous cette mutation, l'aide
`attendre_une_requete_en_cours` ne voit jamais de requête `onboarding_state … FOR UPDATE` en attente — la
requête bloquée est l'`UPDATE` — et **panique** au bout de 10 s : rouge dans les deux issues) ; ne pas rendre `Invariant`
sur `None` (test 7c rouge) ; retirer la garde `company_count == 0` de la branche `(0, false)` du
bootstrap (test 14 rouge : deux sociétés) ; substituer l'erreur du rollback à l'erreur d'origine
(non distinguée par un test : le cas exige un `ROLLBACK` qui échoue — garde par relecture, écrite au
Dev Agent Record) ; **retirer le `close_on_drop()`** (test 8 rouge :
même `CONNECTION_ID()` — la preuve — et, en ceinture, `foreign_key_checks = 0` puisque le chemin d'erreur ne rétablit plus rien) ;
remplacer le `rollback` explicite par un commit (test 8 rouge) ; déplacer `refresh_tokens` de
`RESET_PRESERVED_TABLES` vers les tables vidées (test 10b rouge — règle 1, `refresh_tokens` ne
rejoignant `companies` que par `users`) ; calculer la relation de l'AC 3 par fermeture transitive
ordinaire, traversée de `users` comprise (test 10b rouge — règle 2 vide, alors qu'elle doit désigner
exactement deux tables) ; déplacer `invoices` vers `RESET_PRESERVED_TABLES` (test 10b rouge — règle 3 :
`invoices` référence `contacts`, `journal_entries` et `projects`, vidées) ; écrire `NULL` dans
`address_street` au lieu de `''` (tests 4 et 5 rouges : erreur 1048) ; `is_seed_retryable` rendant
vrai pour toute `Db(_)` (test 13 rouge sur le 1205) ou faux toujours (test 13 rouge sur le 1213). Le
type `SeedAttemptError` sans `From<sqlx::Error>` rend le `?` brut **incompilable** : c'est la garde de
la conversion, le test 13 celle du prédicat. **Angle mort assumé** : l'abandon de la future en plein
essai n'est pas provoqué par un test ; il est couvert **structurellement** par `close_on_drop()`, dont
le test 8 prouve l'appel sur le chemin d'erreur — à écrire au Dev Agent Record.

### Ce que la story ne fait pas

- **L'atomicité des quatre premières validations de `seed_demo`** et la sérialisation de `seed_demo`
  avec la remise à zéro : #538 (qui porte désormais les deux).
- **Les fichiers de `KESH_DOCUMENTS_DIR`** (PDF figés, pièces importées) : non effacés, orphelins mais
  inoffensifs — nommage par empreinte (`document_storage.rs`), donc aucune collision (L-2 de la P4) ;
  le manuel le dit (AC 11, `:1314`).
- **`restore_tables_in_tx`** et sa connexion abandonnée : #540.
- **#435**, **#431** — inchangées. **Le RBAC de `seed-demo`**, **le libellé du dialogue de
  confirmation** : signalés, hors périmètre.
- Aucun changement de **comportement** frontend (un commentaire corrigé, `+layout.svelte:17-19` ; un
  test de composant ajouté), aucune migration.
- **La réparation des installations déjà atteintes par #528 et #542** — au démarrage (#528, #542) et à
  la restauration d'une sauvegarde (#528), par la règle des principaux orphelins posée ici : **15-7b3**
  (C-15-7-40, C-15-7-45, C-15-7-46).
- **Une sortie de la démonstration sans geste de l'exploitant** : choix de sécurité existant, conservé et
  écrit (C-15-7-48) ; la question produit est signalée sur #534 par l'orchestrateur.
- **Les compose de production** (`docker-compose.yml`, `docker-compose.prod.yml`), qui ne transmettent
  pas `KESH_PRODUCTION_RESET` : **15-11a** (#550), mergée avant celle-ci (C-15-7-51).
- **La garde « ligne `onboarding_state` absente »** atteinte par HTTP (L-4 de la P2) : le handler recrée
  la ligne à l'étape 0 avant l'appel ; angle mort écrit, non corrigé.

### References

- Issues #434, #528, #279 (piste de correction « option 1 » : la liste canonique), #542, #538, #540,
  #534 (libellé du refus, sortie de la démonstration), #550 (variables non transmises par les compose de
  production ; story 15-11a), #97 (KF-029).
- `kesh_db::test_fixtures::attendre_une_requete_en_cours` (`test_fixtures.rs:537-591`) pour le test 7b.
- Fiche d'origine `15-7b-trace-demo-et-remise-a-zero.md` (corps vidé ; version complète au commit
  `9bbc52de`) ; fiche sœur `15-7b1-trace-demonstration.md` ; fiches `15-7a1-…`, `15-7a2-…` ; fiche
  index `15-7-trace-onboarding.md`.
- `kesh-db/src/backup.rs` (`TABLES_TO_TRUNCATE`, et `restore_tables_in_tx` `:405-431` pour le
  rétablissement des contrôles FK) ; `docs/MULTI-TENANT-SCOPING-PATTERNS.md` § Pattern 5 ; sqlx-core
  0.8.6 `pool/connection.rs` (`close_on_drop`, `Drop`).
- Passes de la 15-7b : prompts versionnés `15-7b-validate-prompt-p{2,3,4}.md` ; rapports
  `target/gate-logs/15-7b-p{2,3,4}-{R,F}.md` (non versionnés). Passes de cette fiche : prompts versionnés
  `15-7b2-validate-prompt-p{1,2,3,4,5}.md` ; rapports `target/gate-logs/15-7b2-p{1,2,3,4,5}-{R,F}.md` (non versionnés).
  Fiche née du découpage de celle-ci : `15-7b3-reparation-des-installations-atteintes.md`.

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5 (développement, worktree `kesh-15-7b2`, branche `story/15-7b2-remise-a-zero`, base `origin/main` `181efa3c`).

### T0 — la fiche relue contre `181efa3c` (avant tout code)

La fiche date d'avant les 15-13a/b, 15-6c/d, 15-1a-i et 15-14a. Écarts constatés, relevés depuis la source :

1. **Registre des routes** (`crates/kesh-api/tests/audit_route_registry.rs`) : **114** routes, `traced` **107**, `exempt` **5**, `no_matter` **2** (la fiche, recomptée au T0 de la 15-7b1, disait 105 / 5 / 2 sur 112 — la 15-1a-i a ajouté le lettrage et le délettrage manuels). Après cette story : **108 / 4 / 2 = 114**. La seconde colonne, `Rejeu`, porte `onboarding::reset` en `Exemptee(…)` ; le rejeu que pose l'AC 2 vit dans `kesh-seed`, comme celui de `seed_demo` : la route passe à `SansEcritureAuJournal` (patron C-15-7b1-1), point (vi) du doc-comment du registre de « cinq » à « six » routes rejouées quand même ; partition de rejeu `exemptees` 4 → **3**, `sans_ecriture` 89 → **90**, `rejouees` 24 inchangé (C-15-7b2-1).
2. **Manuel d'administration** : « 105 des 112 routes … 105 + 5 + 2 = 112 » (`admin-manual.tex`, § *Journal d'audit*) est **déjà périmé** à `181efa3c` (107 / 114 depuis la 15-1a-i). La cible de l'AC 11 devient « **108 des 114** … 108 + 4 + 2 = 114 ».
3. **Tables** : `TABLES_TO_TRUNCATE` compte toujours **39** tables ; `reset_cleared_tables()` en rendra **33**. Le **lettrage** (15-1a-i) n'a pas de table : deux colonnes de `journal_entry_lines` (`lettering_key`, `lettering_origin`, migration `20261009000001`), vidées avec les lignes ; `letterings.rs` et `letterings_lexical.rs` citent déjà `reset_demo` comme suppression en bloc par `DELETE` — vrai encore (aucun littéral de la remise à zéro ne nomme ces colonnes). La **sauvegarde persistante** (15-13b, `KESH_ADMIN_BACKUP_DIR`, défaut `/data/backup`) est un dossier de fichiers, hors base : la remise à zéro n'y touche pas, comme aux fichiers de `KESH_DOCUMENTS_DIR` (inventaire § 5) — dit au manuel avec eux.
4. **`companies`** : 23 colonnes au squash, soit `id`, `version`, `created_at`, `updated_at` et les **19** de `PERTURBED` — conforme au test 5.
5. **Déclencheurs** : le test 8 ne pose **pas** le premier déclencheur du dépôt — les 15-7a2 (`t_15_7a_fail`) et 15-7b1 (`t_15_7b1_*`) en posent déjà. Sans effet sur le test.
6. **`docs/api-external.md`** (§ *Interblocages*) dit que « l'effacement des données de démonstration, réservé à l'interface d'administration, ne rejoue pas » : faux une fois l'AC 2 livrée — à corriger (site absent de l'AC 10).
7. **Test 13** : la 15-7b1 a déjà posé `is_seed_retryable_accepts_1213_and_only_it` sur une 1213 levée par `SIGNAL`. La fiche demande un **vrai** cycle de verrous : il est écrit en plus, avec `ResetForbidden` ; et le rejeu **de `reset_demo` lui-même** est prouvé de bout en bout par un déclencheur qui lève une 1213 une fois (patron C-15-7b1-2), ce que la fiche déclarait angle mort (C-15-7b2-2).
8. **Compose (T8)**, contrôle fait dès le T0 : `grep -n "PRODUCTION_RESET" docker-compose.yml docker-compose.prod.yml` rend `docker-compose.prod.yml:148` et `docker-compose.yml:128` — une ligne par fichier, la 15-11a est mergée.
9. **Sites relocalisés par le texte** : `admin-manual.tex` `:730` (cellule `KESH\_PRODUCTION\_RESET`), `:1021` (« company provisoire »), `:1308` (matrice), `:1368` (`\item \textbf{Parcours}`), `:2088` (105 / 112), `:2121` (« efface encore »), `:2275` (réserve OLICo, \#434) ; `user-manual.tex` `:190` (§ Chemin A, renvoi au bouton avec sa condition — déjà écrit par la 15-7b1), `:2172` (« Deux familles »), `:2408` (glossaire). `docs/MULTI-TENANT-SCOPING-PATTERNS.md` : ligne `reset` `:324`, *Known Risk — KF-002-H-002* `:343-345`.
10. **Test 7 de la 15-7a2** : `language_without_company_traces_company_created` (`onboarding_audit_e2e.rs`) asserte encore l'id mort — remplacé par le test 6c.
11. **Pattern 5** : la liste d'exceptions « Deny list » (`*(none)*`) que vise l'AC 10 **n'existe plus** dans `docs/MULTI-TENANT-SCOPING-PATTERNS.md` ; l'exception s'écrit en note, sur le patron des notes voisines (C-15-7b2-3, constaté au développement).

### Debug Log References

Journaux non versionnés, dans `/home/gcorbaz/devel/kesh-gate-logs/` : `15-7b2-mutations.log` (banc de mutations), `15-7b2-gate-backend-1.log` (gate backend complet), `15-7b2-e2e-1.log` (Playwright complet), `15-7b2-backend-e2e.log` et `15-7b2-backend-e2e-flag.log` (backends E2E), `15-7b2-make-fr.log`, `15-7b2-make-fr-2.log` (PDF). Script du banc : scratchpad de session (`mutations.py`), non versionné — sa liste est recopiée ci-dessous.

### Completion Notes List

**Ce qui est livré** (AC 2 à 13) :

- `kesh_seed::reset_demo(pool, actor, production_reset_allowed) -> Result<ResetOutcome, SeedError>` : `retry_with` (`DEFAULT_MAX_DEADLOCK_ATTEMPTS`, `is_seed_retryable` réemployé), un essai = connexion dédiée `close_on_drop()` + transaction (`sqlx::Connection::begin`), `SET FOREIGN_KEY_CHECKS = 0` dans la transaction, `lock_state_in_tx` puis les trois gardes **sous ce verrou**, `companies FOR UPDATE` (0 ⇒ `insert_stub`, 1 ⇒ conservée, >1 ⇒ `Invariant`), relevé verrouillant d'`audit_log`, vidage de `reset_cleared_tables()` (garde `rows_affected` sur `audit_log`), `reset_to_stub_in_tx`, `reattach_orphan_principals_in_tx(Some)`, `reset_state_in_tx`, `installation.reset`, `SET FOREIGN_KEY_CHECKS = 1`, commit ; rollback explicite dont l'erreur est journalisée, jamais substituée. `SeedError::ResetForbidden`, `SeedAttemptError::ResetForbidden`, `ResetOutcome` à champs publics. Toute erreur sqlx de l'essai passe par `map_db_error` (le type l'impose).
- `kesh-db` : `companies::{STUB_COMPANY_NAME, STUB_COMPANY_ADDRESS, insert_stub, reset_to_stub_in_tx, reattach_orphan_principals_in_tx, RevokedApiKey, OrphanPrincipals}`, `onboarding::reset_state_in_tx`, `backup::{RESET_PRESERVED_TABLES, reset_cleared_tables}` (33 tables vidées) ; littéral du stub de `test_fixtures.rs` remplacé par la constante.
- Handler `reset` : `Extension(current_user)`, plus de transaction ni de garde, `get_or_init_state` avant et relecture après conservés, correspondance `SeedError` → `AppError` complète (400 / 403 / `Database` / `Internal`). Branche « aucune société » de `language` : `insert_stub` + rattachement, détails `users_repointed`, `api_keys_revoked`, `api_keys_repointed` dans `company.created`. Bootstrap : `insert_stub_company` enveloppe `insert_stub`, cas `(0, false)` gardé par `company_count == 0` (#542).
- Action `installation.reset`, quatre catalogues (libellés de l'AC 9) ; registre 108 / 4 / 2 = 114, rejeu 24 / 3 / 90 = 117 (C-15-7b2-1).
- Doc-comments (AC 10) : grep exécuté (sortie triée ci-dessous) ; Pattern 5 (ligne `reset`, note d'exception, *Known Risk*) ; `.env.example`, `docker-compose.dev.yml` ; `docs/api-external.md` (§ *Interblocages*, écart T0 n° 6).
- Manuels (AC 11), PDF par `make -B fr`, contrôlés aplatis ; CHANGELOG `[0.13.0]` (AC 12) ; commentaire du layout d'onboarding.

**Grep de l'AC 10** (exécuté au commit `d64a9545`, motif de la fiche sur `crates docs/MULTI-TENANT-SCOPING-PATTERNS.md .env.example docker-compose*.yml frontend/src`) — sites traités : `helpers.rs:55-58`, `lib.rs:306-313`, `errors.rs:282`, `accounts.rs:1134`, `audit_log.rs:16-25`, `journal_entries.rs:73-74`, `entities/company.rs:174`, `repositories/onboarding.rs` (`delete_state`), `routes/onboarding.rs` (doc d'`is_stub`, doc et corps de `reset`, branche « aucune société »), `auth/bootstrap.rs` (constantes, cas 1, doc d'`ensure_admin_user`), `+layout.svelte`, `.env.example`, `docker-compose.dev.yml`, Pattern 5. Sites légitimes laissés : `config.rs` et `configuration_transmise.rs` (« placeholder » au sens des gabarits de `.env`), `onboarding_e2e.rs` (drapeau posé par les tests), `routes/onboarding.rs:50` (doc d'`env_flag_enabled`), `routes/onboarding.rs:712-723`, `reconciliation.rs:4256`, `kesh-db/Cargo.toml:23` et `retry.rs` (KF-002-H-002 = #43, historique de `finalize`), `journal_entries.rs:1894` (historique, toujours vrai : `reset_demo` n'emprunte pas `delete_all_by_company`), `letterings.rs:19`, `letterings_lexical.rs:19` (vrais : le vidage est un `DELETE`), `audit_log.rs:414, :449` (`DELETE … WHERE id = ?` des tests), `bootstrap.rs:139` (#434 / #435, historique de la 25-1b), `input.svelte` (« aria-invalid », faux positif de `(none)`), les trois compose (une ligne de transmission chacun).

**Gates sur l'état REBASÉ — gate de référence** (branche rebasée sur `origin/main` `056997b0`, 15-1a-ii mergée ; HEAD `19ebf85d`, dernier commit de code `5fe1f918` ; bases `kesh_157b2` et `kesh_e2e_157b2` reconstruites avant) : backend `scripts/test-fast.sh` **3197 / 3197**, 4 ignorés (3178 au gate de la 15-1a-ii + 19 nets) ; frontend `check` 0 erreur, `lint-i18n-ownership` PASS, Vitest **1164 / 1164** (1161 + 3), `build` vert ; E2E complet **247 passés, 7 échecs, 19 ignorés** — exactement les 7 KF-029, aucun hors liste. Rebase : conflits résolus par union (`admin-manual.tex`, réserve OLICo : délettrage de la 15-1a-ii + gestes de session seuls ; `sprint-status.yaml`, ligne `last_updated` renumérotée (62)) ; PDF régénérés sur l'état rebasé et recontrôlés aplatis (mêmes présences et absences qu'avant, Overfull 55 / 27, 0 référence indéfinie). tmpfs 1,5 Go en fin de session (1,3 Go au début), sur 8 Go.

**Gates avant rebase — exécutés au commit de code `d64a9545`** (base `181efa3c`, historique) (les commits suivants ne portent que la fiche, le registre des choix, un renvoi du manuel utilisateur et ses PDF) :

- Gate backend complet `scripts/test-fast.sh` (fmt + clippy `-D warnings` + nextest, base `kesh_157b2` reconstruite avant) : **3180 / 3180**, 4 ignorés (`origin/main` 181efa3c : 3161 au gate de la 15-14a ; +19 nets, recomptés : `onboarding_audit_e2e.rs` 33 → 46 — 14 ajoutés, 1 retiré —, `companies_repository.rs` 19 → 22, `backup.rs` 10 → 12, `bootstrap.rs` 9 → 10).
- Frontend : `npm run check` 0 erreur (27 avertissements, préexistants, aucun dans un fichier touché), `lint-i18n-ownership` PASS, `test:unit` **1161 / 1161** (`onboarding.svelte.test.ts` 7 → 8, `onboarding-layout.test.ts` 0 → 2), `build` vert.
- E2E complet (base `kesh_e2e_157b2` reconstruite, port 3019, recette de `docs/testing.md` complète, drapeau **absent**) : **246 passés, 8 échecs, 19 ignorés**. Les 8 jugés fichier par fichier : les **7 KF-029** (#97 : `mode-expert:26`, `:41`, `onboarding-path-b:65`, `:92`, `onboarding:57`, `:77`, `:150`) et `sidebar-navigation:75` (pollution connue, #424), **vert rejoué seul**. Aucun hors liste.
- **Cause de `onboarding.spec.ts:57` vérifiée** (T8, à signaler sur #97) : même backend relancé **avec** `KESH_PRODUCTION_RESET=true`, le test rejoué seul **passe** (1 passed). Le montage E2E de `docs/testing.md` ne pose pas le drapeau ; la remise à zéro d'une démonstration à l'étape 3 y est refusée en 403, ce que le test ne prévoit pas.
- Après la retouche du renvoi du manuel utilisateur (docs seuls) : `textes_coherents` et `configuration_transmise` rejoués, **36 / 36**.
- Contrôle T8 des compose : `grep -n "PRODUCTION_RESET" docker-compose.yml docker-compose.prod.yml` ⇒ `docker-compose.prod.yml:148:      KESH_PRODUCTION_RESET: ${KESH_PRODUCTION_RESET:-}` et `docker-compose.yml:128:      KESH_PRODUCTION_RESET: ${KESH_PRODUCTION_RESET:-}` — une ligne par fichier. `docker compose config` **non exécuté** ici (la 15-11a l'a mesuré ; la transmission se lit sur la ligne `environment:`).
- PDF : `make -B fr`, 0 référence indéfinie ; Overfull 55 (admin) et 27 (utilisateur), **aucun** sur une ligne modifiée (croisé avec `git diff -U0`) ; contrôle aplati (`pdftotext | tr '\n' ' ' | tr -s ' '`) : la cellule `KESH_PRODUCTION_RESET` se lit entière, de « Autorise » à « sans casse » ; « 108 des 114 », « 108 + 4 + 2 = 114 », « efface la table et y inscrit son geste », « Sortir de la démonstration », « le trou qui précède cette entrée s’explique par elle » présents ; « efface encore », « deux familles », « 105 des 112 » absents des deux PDF. La brochure, régénérée sans changement de source, est remise à sa version de `main`.
- tmpfs MariaDB (lecture seule) : 1,3 Go avant, 1,3 Go après (8 Go).

**Mutations — 31, toutes rouges, chacune par un test en échec** (aucune par défaut de compilation ; fichier restauré depuis `HEAD` et touché après chacune ; journal `15-7b2-mutations.log`) : M1 liste en dur de huit tables (test 4) ; M2 `DELETE FROM companies` au lieu du stub en place (test 5) ; M3 `website = NULL` retiré (test 5) ; M4 `website` retiré de `PERTURBED` (test 5, inventaire) ; M5 langue sans société sans rattachement (6c) ; M6 `Invariant` sur zéro société (6b) ; M7 plus d'une société acceptée (6d, deux tests) ; M8 repointer sans révoquer (6f), M8b idem par HTTP (6c) ; M9 révoquer sans repointer (6f (i)) ; M10 filtre `revoked_at IS NULL` retiré (6f (i)) ; M11 prédicat d'orphelin retiré de la sélection des clés (test 4), M11b idem (6f (i)) ; M12 pas de révocation sans cible (6f (ii)) ; M13 lecture d'état non verrouillante (7b) ; M14 pas d'`Invariant` sur ligne absente (7c) ; M15 garde `company_count == 0` retirée (14) ; M16 `close_on_drop()` retiré (8) ; M17 rollback remplacé par commit (8) ; M18 `refresh_tokens` vidée (10b) ; M19 fermeture transitive ordinaire (10b, règle 2) ; M20 `invoices` conservée (10b, règle 3) ; M21 `NULL` dans `address_street` (5) ; M22 prédicat vrai pour toute `Db(_)` (13) ; M23 prédicat toujours faux (13 et 13b) ; M24 `retry_with` retiré (13b) ; M25 remise à zéro sans rattachement (6a) ; M26 `ResetForbidden` rendu en 400 (7) ; M27 garde « production au-delà de l'étape 2 » retirée (7) ; M28 garde du drapeau retirée (7) ; M29 état non remis à zéro (4).

**Angles morts, écrits** :

- L'interblocage **naturel** dans le corps de `reset_demo` n'est pas provoqué ; le rejeu est prouvé sur une 1213 levée **une fois** par déclencheur (13b, C-15-7b2-2), le prédicat sur une 1213 née d'un vrai cycle InnoDB (13).
- L'abandon de la future en plein essai n'est pas provoqué ; il est couvert **structurellement** par `close_on_drop()`, dont le test 8 prouve l'effet sur le chemin d'erreur (identifiant de connexion changé).
- La substitution de l'erreur du rollback à l'erreur d'origine n'est distinguée par aucun test (il faudrait un `ROLLBACK` qui échoue) : garde par relecture — `reset_attempt` rend `Err(e)` d'origine et journalise `rb`.
- La garde `rows_affected` du `DELETE FROM audit_log` n'est pas testée (non provocable) ; la lecture verrouillante agrégée reste une conjecture non vérifiée contre MariaDB (F3-6), la sûreté n'en dépendant pas.
- Garde « ligne `onboarding_state` absente » inatteignable par HTTP (L-4 de la P2) : le handler recrée la ligne à l'étape 0, non démonstration, et une installation de production dont la ligne aurait disparu serait vidée sans drapeau. Préexistant, portée élargie par #279 ; non corrigé ici.
- Tables à réglages par défaut (AC 3, vérification table par table) : `company_dunning_settings` est get-or-create, `dunning_levels` paresseux (gardé par `seeded_at`, colonne de `company_dunning_settings`, vidée avec eux : le réamorçage a lieu), `company_invoice_settings` et `vat_rates` recréés par `finalize` ou par `seed_demo` — une société remise à zéro repart comme une société neuve ; aucun cas contraire relevé. `email_templates` : surcharges seulement, l'absence de ligne retombe sur le texte par défaut (`repositories/email_templates.rs`, doc du module).
- Les fichiers de `KESH_DOCUMENTS_DIR` et les sauvegardes de `KESH_ADMIN_BACKUP_DIR` restent sur le disque (dit au manuel).
- `onboarding.spec.ts:57` reste un échec attendu du montage E2E documenté (sans drapeau) ; il passe avec le drapeau (ci-dessus).

**Choix consignés** : C-15-7b2-1 (registre, rejeu), C-15-7b2-2 (rejeu prouvé de bout en bout), C-15-7b2-3 (exception au Pattern 5 en note), C-15-7b2-4 (montage du test 4).

### Revue de code P1 — remédiation (sans code)

**A-1 = E-3 (MEDIUM)** — corrigé : `admin-manual.tex` (`\item \textbf{Parcours}`, paragraphe #528) borne la réparation à une installation qui compte **aucune ou une** société, et dit qu'à deux sociétés ou plus — réelles ou provisoires — le bouton répond par une erreur interne sans rien effacer (réparation : 15-7b3). Même bornage au CHANGELOG et à la ligne de l'AC 11. Grep de la valeur (`quel que soit`, `nombre de sociétés`) sur tout le dépôt suivi : seuls sites de la promesse, `admin-manual.tex:1374` et la fiche `:543` (corrigés) ; la ligne du Change Log P4 (`R4-5`) est historique ; les autres occurrences (« quel que soit le rôle », « le mode »…) sont sans rapport ; aucun site dans les quatre catalogues ni dans `user-manual.tex`. PDF : `make -B fr` exit 0, 0 référence indéfinie ; aplati, « quel que soit son nombre » : 0 occurrence, « si elle compte aucune ou une société » et « deux sociétés ou plus » présents ; brochure et manuel utilisateur, régénérés sans changement de source, remis à leur version de `HEAD`. Gardes de texte et de configuration rejouées après la correction (`textes_coherents`, `configuration_transmise`) : **36 / 36**.

**A-4** — Overfull comptés **par passe** dans `kesh-gate-logs/15-7b2-make-fr-4.log` : `admin-manual` 55 et 55, `user-manual` 27 et 27, `marketing-brochure` 4 et 4. Le « 35 » relevé par la lentille additionnait la brochure à la seconde passe du manuel utilisateur.

**A-2** — `docker compose config` exécuté en lecture (aucun `up`, aucun conteneur créé : `docker ps | grep -c 157b2` → 0), `.env` factice passé par `--env-file` (`MARIADB_ROOT_PASSWORD`, `MARIADB_PASSWORD`, `KESH_JWT_SECRET` factices) :

```
$ docker compose -f docker-compose.yml --env-file <factice, KESH_PRODUCTION_RESET=true> config | grep PRODUCTION_RESET
37:      KESH_PRODUCTION_RESET: "true"
$ docker compose -f docker-compose.prod.yml --env-file <factice, KESH_PRODUCTION_RESET=true> config | grep PRODUCTION_RESET
30:      KESH_PRODUCTION_RESET: "true"
$ idem docker-compose.yml, variable absente du .env :
37:      KESH_PRODUCTION_RESET: ""
```

**A-3** — grep de l'AC 10 rejoué sur l'état rebasé (HEAD `e3368ba7`, code identique à `5fe1f918` : `git diff --stat 5fe1f918 HEAD -- crates frontend/src` vide). Sortie triée, coupée à 150 colonnes : **410 lignes**, dont **306** portées par le motif `placeholder` (attributs HTML, catalogues, gabarits de `.env` de `config.rs` — sans rapport avec la story) ; sortie intégrale versée à `kesh-gate-logs/15-7b2-ac10-grep-rebase.txt`. Les **104** autres, collées ci-dessous, sont toutes traitées par la story ou déclarées légitimes plus haut (aucun site neuf) :

```
crates/kesh-api/src/auth/bootstrap.rs:139:            // n'étant pas une route, il échappait aussi aux issues #434 et #435.
crates/kesh-api/src/errors.rs:283:    /// sans le drapeau `KESH_PRODUCTION_RESET` (sortir d'une démonstration
crates/kesh-api/src/lib.rs:308:        // `!is_demo && > 2`, le drapeau `KESH_PRODUCTION_RESET`. *Un état se
crates/kesh-api/src/lib.rs:313:        // Story 15-7b2 (#434), y INSCRIT son geste (`installation.reset`), en
crates/kesh-api/src/routes/onboarding.rs:253:/// geste** au journal d'audit (Story 15-7b2, #434, #279, #528).
crates/kesh-api/src/routes/onboarding.rs:255:/// Toute l'opération est dans `kesh_seed::reset_demo` : **une** transaction
crates/kesh-api/src/routes/onboarding.rs:260:/// avec le drapeau ; au-delà de l'étape 2 sans `KESH_PRODUCTION_RESET` ⇒ `403`
crates/kesh-api/src/routes/onboarding.rs:266:/// `KESH_PRODUCTION_RESET` : une démonstration est toujours à l'étape 3, si
crates/kesh-api/src/routes/onboarding.rs:272:/// verrou rendrait `None`, que `reset_demo` refuse en `Invariant`) ; la réponse
crates/kesh-api/src/routes/onboarding.rs:280:    let production_reset_allowed = env_flag_enabled("KESH_PRODUCTION_RESET");
crates/kesh-api/src/routes/onboarding.rs:282:    kesh_seed::reset_demo(
crates/kesh-api/src/routes/onboarding.rs:292:        // Inatteignable par `reset_demo` (toute erreur sqlx y passe par
crates/kesh-api/src/routes/onboarding.rs:50:/// Used for `KESH_PRODUCTION_RESET` and any future opt-in env flag where a
crates/kesh-api/src/routes/onboarding.rs:712:/// KF-002-H-002 (#43) closed 2026-05-03 : la fonction est enveloppée dans
crates/kesh-api/src/routes/onboarding.rs:723:    // KF-002-H-002 (#43) : la closure ci-dessous est rappelée intégralement
crates/kesh-api/src/routes/reconciliation.rs:4256:/// `onboarding::finalize` (KF-002-H-002, #43). ⚠️ Le prédicat porte sur
crates/kesh-api/tests/audit_route_registry.rs:136://!   Story 15-7b1, #434 — `onboarding::seed_demo`, dont la dernière
crates/kesh-api/tests/audit_route_registry.rs:148://!   ligne. Et — Story 15-7b2, #434 — `onboarding::reset`, dont chaque essai
crates/kesh-api/tests/audit_route_registry.rs:149://!   est une transaction unique rejouée **dans `kesh-seed`** (`reset_demo`,
crates/kesh-api/tests/audit_route_registry.rs:646:         de configuration de l'installation (15-7a2, #434), plus le peuplement de \
crates/kesh-api/tests/audit_route_registry.rs:647:         démonstration (15-7b1, #434), plus le lettrage et le délettrage manuels \
crates/kesh-api/tests/audit_route_registry.rs:648:         (15-1a-i, #518), plus la remise à zéro (15-7b2, #434)"
crates/kesh-api/tests/configuration_transmise.rs:164:    ("KESH_PRODUCTION_RESET", Compose::Y),
crates/kesh-api/tests/configuration_transmise.rs:165:    ("KESH_PRODUCTION_RESET", Compose::P),
crates/kesh-api/tests/configuration_transmise.rs:1865:        "KESH_PRODUCTION_RESET",
crates/kesh-api/tests/onboarding_audit_e2e.rs:1252:// Story 15-7b1 (#434) — le chargement de la démonstration laisse sa trace
crates/kesh-api/tests/onboarding_audit_e2e.rs:1764:// Story 15-7b2 (#434, #279, #528) — la remise à zéro laisse sa trace, vide
crates/kesh-api/tests/onboarding_audit_e2e.rs:1774:/// `POST /api/v1/onboarding/reset` avec `KESH_PRODUCTION_RESET` posé le temps
crates/kesh-api/tests/onboarding_audit_e2e.rs:1778:    let prev = std::env::var("KESH_PRODUCTION_RESET").ok();
crates/kesh-api/tests/onboarding_audit_e2e.rs:1781:    unsafe { std::env::set_var("KESH_PRODUCTION_RESET", "true") };
crates/kesh-api/tests/onboarding_audit_e2e.rs:1785:            Some(v) => std::env::set_var("KESH_PRODUCTION_RESET", v),
crates/kesh-api/tests/onboarding_audit_e2e.rs:1786:            None => std::env::remove_var("KESH_PRODUCTION_RESET"),
crates/kesh-api/tests/onboarding_audit_e2e.rs:1795:    unsafe { std::env::remove_var("KESH_PRODUCTION_RESET") };
crates/kesh-api/tests/onboarding_audit_e2e.rs:1983:/// peuplée : **chaque** table vidée l'est (#279, la classe entière), la piste
crates/kesh-api/tests/onboarding_audit_e2e.rs:1://! La piste de contrôle de l'installation — Story 15-7a2 (#434) pour la
crates/kesh-api/tests/onboarding_audit_e2e.rs:2071:            assert_eq!(after[t], 0, "#279 : {t} doit être vidée");
crates/kesh-api/tests/onboarding_audit_e2e.rs:2537:/// A tient `onboarding_state` ; `reset_demo` est vu bloqué sur son `FOR
crates/kesh-api/tests/onboarding_audit_e2e.rs:2567:    let handle = tokio::spawn(async move { kesh_seed::reset_demo(&p, actor, true).await });
crates/kesh-api/tests/onboarding_audit_e2e.rs:2575:        "reset_demo doit attendre le verrou d'état"
crates/kesh-api/tests/onboarding_audit_e2e.rs:2591:/// Test 7c (15-7b2, AC 2 ; R6 de la P1) — ligne d'état absente, `reset_demo`
crates/kesh-api/tests/onboarding_audit_e2e.rs:2600:    let result = kesh_seed::reset_demo(&pool, (admin_user_id(&pool).await, None), true).await;
crates/kesh-api/tests/onboarding_audit_e2e.rs:2654:    let result = kesh_seed::reset_demo(&pool1, actor, true).await;
crates/kesh-api/tests/onboarding_audit_e2e.rs:2677:    let outcome = kesh_seed::reset_demo(&pool1, actor, true).await.unwrap();
crates/kesh-api/tests/onboarding_audit_e2e.rs:2691:    sqlx::raw_sql("DELETE FROM audit_log")
crates/kesh-api/tests/onboarding_audit_e2e.rs:2769:/// Test 13b (15-7b2, C-15-7b2-2) — le rejeu de `reset_demo` **lui-même**, de
crates/kesh-api/tests/onboarding_audit_e2e.rs:2://! production, Story 15-7b1 (#434) pour le chargement de la démonstration,
crates/kesh-api/tests/onboarding_audit_e2e.rs:3://! Story 15-7b2 (#434, #279, #528) pour la remise à zéro.
crates/kesh-api/tests/onboarding_e2e.rs:395:    // requires KESH_PRODUCTION_RESET=1 to allow (demo deployment opt-in).
crates/kesh-api/tests/onboarding_e2e.rs:396:    let prev = std::env::var("KESH_PRODUCTION_RESET").ok();
crates/kesh-api/tests/onboarding_e2e.rs:398:        std::env::set_var("KESH_PRODUCTION_RESET", "1");
crates/kesh-api/tests/onboarding_e2e.rs:411:            Some(v) => std::env::set_var("KESH_PRODUCTION_RESET", v),
crates/kesh-api/tests/onboarding_e2e.rs:412:            None => std::env::remove_var("KESH_PRODUCTION_RESET"),
crates/kesh-api/tests/onboarding_e2e.rs:467:/// P6-L5 — Positive path: when KESH_PRODUCTION_RESET=1 and is_demo=true at step > 2,
crates/kesh-api/tests/onboarding_e2e.rs:495:    let prev = std::env::var("KESH_PRODUCTION_RESET").ok();
crates/kesh-api/tests/onboarding_e2e.rs:499:        std::env::set_var("KESH_PRODUCTION_RESET", "true");
crates/kesh-api/tests/onboarding_e2e.rs:513:            Some(v) => std::env::set_var("KESH_PRODUCTION_RESET", v),
crates/kesh-api/tests/onboarding_e2e.rs:514:            None => std::env::remove_var("KESH_PRODUCTION_RESET"),
crates/kesh-api/tests/onboarding_e2e.rs:521:        "reset() with KESH_PRODUCTION_RESET=true must succeed for is_demo=true at step 5"
crates/kesh-api/tests/onboarding_e2e.rs:529:/// regardless of KESH_PRODUCTION_RESET. P6-L8: distinct ONBOARDING_RESET_FORBIDDEN
crates/kesh-db/Cargo.toml:23:# KF-002-H-002 (#43) : `tokio::time::sleep` pour le backoff entre retries
crates/kesh-db/src/backup.rs:110:/// Les tables que la remise à zéro **vide** — Story 15-7b2 (AC 3, #279) :
crates/kesh-db/src/backup.rs:88:/// Les tables que la **remise à zéro** d'une installation (`kesh_seed::reset_demo`)
crates/kesh-db/src/backup.rs:89:/// **conserve** — Story 15-7b2 (AC 3, choix C-15-7-10, #279).
crates/kesh-db/src/repositories/accounts.rs:1135:/// (`kesh_seed::reset_demo`) ne l'emprunte pas : elle vide les tables de
crates/kesh-db/src/repositories/audit_log.rs:16://! 2. **`reset_demo`** — `DELETE FROM audit_log` non scopé, sur une route montée
crates/kesh-db/src/repositories/audit_log.rs:20://!    refusé inconditionnellement, drapeau `KESH_PRODUCTION_RESET` posé ou non —,
crates/kesh-db/src/repositories/audit_log.rs:21://!    et, depuis la Story 15-7b2 (#434), **elle y inscrit son geste** :
crates/kesh-db/src/repositories/audit_log.rs:417:        sqlx::query("DELETE FROM audit_log WHERE id = ?")
crates/kesh-db/src/repositories/audit_log.rs:452:        sqlx::query("DELETE FROM audit_log WHERE id = ?")
crates/kesh-db/src/repositories/companies.rs:150:/// (`kesh_seed::reset_demo`, Story 15-7b2), qui le recrée **en place** ou
crates/kesh-db/src/repositories/companies.rs:299:/// Appelants : `kesh_seed::reset_demo` (`Some`), la branche « aucune société »
crates/kesh-db/src/repositories/journal_entries.rs:2014:/// ⚠️ **Ce doc-comment annonçait « utilisé par `reset_demo` » — c'était faux**, et
crates/kesh-db/src/repositories/journal_entries.rs:2015:/// cette erreur a coûté une passe de revue à la Story 24-4a. `reset_demo`
crates/kesh-db/src/repositories/journal_entries.rs:75://! et `reset_demo` de `kesh-seed` (remise à zéro de l'installation, qui vide
crates/kesh-db/src/repositories/letterings.rs:19://! (`journal_entries::delete_all_by_company`, `reset_demo` de `kesh-seed`).
crates/kesh-db/src/repositories/onboarding.rs:246:/// (`kesh_seed::reset_demo`) remet l'état à zéro **en place**
crates/kesh-db/tests/letterings_lexical.rs:19://! `reset_demo`) sont des `DELETE`, qui emportent les lignes avec leurs marques.
crates/kesh-seed/src/lib.rs:190:    // it is committed before companies::update runs, so a concurrent reset_demo
crates/kesh-seed/src/lib.rs:40:    /// au-delà de l'étape 2 sans `KESH_PRODUCTION_RESET`. Gardes évaluées
crates/kesh-seed/src/lib.rs:441:/// Story 15-7b2 (#434, #279, #528 ; choix C-15-7-5, C-15-7-9 à 11, C-15-7-22,
crates/kesh-seed/src/lib.rs:462:///    canonique (le `DELETE FROM audit_log` doit effacer exactement le compte
crates/kesh-seed/src/lib.rs:484:pub async fn reset_demo(
crates/kesh-seed/src/lib.rs:490:        "kesh_seed::reset_demo",
crates/kesh-seed/src/lib.rs:5://! `POST /api/v1/onboarding/reset` ([`reset_demo`], qui remet l'installation à
crates/kesh-seed/src/lib.rs:504:/// Un essai de [`reset_demo`] : connexion dédiée fermée à sa libération,
crates/kesh-seed/src/lib.rs:524:                tracing::warn!("reset_demo : annulation de l'essai en échec : {rb}");
crates/kesh-seed/src/lib.rs:531:/// Le corps d'un essai de [`reset_demo`] — cf. son doc-comment. Ne commite
crates/kesh-seed/src/lib.rs:545:        DbError::Invariant("onboarding_state absent sous verrou pendant reset_demo".into())
crates/kesh-seed/src/lib.rs:567:                "Expected at most 1 company for reset_demo, found {}",
crates/kesh-seed/src/lib.rs:57:/// Partagé avec la remise à zéro ([`reset_demo`], Story 15-7b2), qui l'étend
crates/kesh-seed/src/lib.rs:583:    // 4. Le vidage, dérivé de la liste canonique (#279).
crates/kesh-seed/src/lib.rs:85:/// essai de [`reset_demo`] : un interblocage MariaDB (1213), et lui seul —
docker-compose.dev.yml:32:      KESH_PRODUCTION_RESET: ${KESH_PRODUCTION_RESET:-}
docker-compose.prod.yml:148:      KESH_PRODUCTION_RESET: ${KESH_PRODUCTION_RESET:-}
docker-compose.yml:128:      KESH_PRODUCTION_RESET: ${KESH_PRODUCTION_RESET:-}
docs/MULTI-TENANT-SCOPING-PATTERNS.md:324:| `kesh_seed::reset_demo` (`POST /onboarding/reset`, Story 15-7b2) | **One transaction per attempt**: **onbo
docs/MULTI-TENANT-SCOPING-PATTERNS.md:325:| `kesh_seed::seed_demo` (`POST /onboarding/seed-demo`) | **First four steps**, each in its own transaction
docs/MULTI-TENANT-SCOPING-PATTERNS.md:343:- **`POST /onboarding/reset` — exception to the Global Lock Order (Story 15-7b2, choice C-15-7-22).** After
docs/MULTI-TENANT-SCOPING-PATTERNS.md:347:### Known Risk — KF-002-H-002 (resolved 2026-05-03)
docs/MULTI-TENANT-SCOPING-PATTERNS.md:349:**Issue:** `seed_demo` (for its **first four steps** — count-validation, `companies::update`, `bulk_create_f
.env.example:340:# KESH_PRODUCTION_RESET=
frontend/src/lib/components/ui/input/input.svelte:28:			"dark:bg-input/30 border-input focus-visible:border-ring focus-visible:ring-ring/50 aria-inval
frontend/src/lib/components/ui/input/input.svelte:41:			"dark:bg-input/30 border-input focus-visible:border-ring focus-visible:ring-ring/50 aria-inval
frontend/src/lib/features/bank-import/BankProfileSelector.svelte:50:	<option value="">— {i18nMsg('bank-import-labels-bank-profile-auto-detect-placehol
```

**Dette — LOW de code, non corrigés ici** (décision de l'orchestrateur, C-15-7b2-6 ; aucun fichier de code ni de test touché par la remédiation) :

| finding | objet | motif de la dette | suite |
|---|---|---|---|
| B-1 | un écart de comptage d'`audit_log` rend `Invariant` (500), non rejoué | sûr (rien d'effacé) ; la lecture verrouillante agrégée est une conjecture déjà écrite (F3-6) | à rejouer si l'écart est un jour observé |
| B-2 | deux sociétés ou plus : 500 générique | AC 2.2 ; la réparation et un refus lisible relèvent de la 15-7b3 | 15-7b3 |
| B-3 | les clés d'API saines survivent, actives | choix de la fiche (AC 3, `RESET_PRESERVED_TABLES`), dit au manuel (« revoyez-les ») | à reconsidérer si la sortie de démonstration devient automatique |
| B-4 | test 10 vrai en partie par construction | écrit dans son doc-comment ; la garde est le test 10b | — |
| B-5 | `onboarding::delete_state` sans appelant de production | assumé par la fiche (AC 2 étape 6) ; sert les tests 7c et `onboarding_repository` | — |
| B-6 = E-6 | tests qui mutent l'environnement du processus | patron hérité d'`onboarding_e2e.rs` ; sûr sous nextest (un processus par test) et sous la CI (`--test-threads=1`) | passer le drapeau par `Config` dans `AppState` |
| E-1 | 403 de sortie de démonstration : toast qui accuse le rôle, clé `error-onboarding-reset-forbidden` absente des quatre catalogues (repli français codé en dur) | défaut antérieur, déjà nommé au manuel et au CHANGELOG | **#534** |
| E-2 | aucun test navigateur de la recette de sortie | `onboarding.spec.ts:57` est KF-029 ; il passe avec le drapeau (mesuré au développement) | #97 (poser le drapeau au montage E2E) |
| E-4 | à l'étape 0-2, aucune garde de présence de données | comportement hérité, conservé (décision de l'orchestrateur) ; angle mort « ligne d'état absente » déjà écrit | arbitrage de Guy |
| E-5 | `installation.reset` ne dit pas ce qui survit (utilisateurs, clés actives) | champ d'audit = code ; non bloquant | story ultérieure |
| E-7 | dialogue de confirmation (`demo-reset-confirm-body`) muet sur le journal effacé et les principaux conservés | texte de catalogue, non de manuel | **#534** |

### File List

- `.env.example`
- `CHANGELOG.md`
- `_bmad-output/implementation-artifacts/15-7b2-remise-a-zero.md`
- `_bmad-output/implementation-artifacts/epic-15-choix-autonomes.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `crates/kesh-api/src/audit_labels.rs`
- `crates/kesh-api/src/auth/bootstrap.rs`
- `crates/kesh-api/src/errors.rs`
- `crates/kesh-api/src/helpers.rs`
- `crates/kesh-api/src/lib.rs`
- `crates/kesh-api/src/routes/onboarding.rs`
- `crates/kesh-api/tests/audit_route_registry.rs`
- `crates/kesh-api/tests/onboarding_audit_e2e.rs`
- `crates/kesh-db/src/backup.rs`
- `crates/kesh-db/src/entities/company.rs`
- `crates/kesh-db/src/repositories/accounts.rs`
- `crates/kesh-db/src/repositories/audit_log.rs`
- `crates/kesh-db/src/repositories/companies.rs`
- `crates/kesh-db/src/repositories/journal_entries.rs`
- `crates/kesh-db/src/repositories/onboarding.rs`
- `crates/kesh-db/src/test_fixtures.rs`
- `crates/kesh-db/tests/companies_repository.rs`
- `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`
- `crates/kesh-seed/src/lib.rs`
- `docker-compose.dev.yml`
- `docs/MULTI-TENANT-SCOPING-PATTERNS.md`
- `docs/api-external.md`
- `docs/manual/fr/admin-manual.tex`, `docs/manual/fr/admin-manual.pdf`
- `docs/manual/fr/user-manual.tex`, `docs/manual/fr/user-manual.pdf`
- `frontend/src/lib/features/onboarding/onboarding.svelte.test.ts`
- `frontend/src/routes/onboarding/+layout.svelte`
- `frontend/src/routes/onboarding/onboarding-layout.test.ts` (nouveau)

## Change Log

- 2026-10-08 — **Née du découpage de la 15-7b** à la passe de validation P4 (choix C-15-7-31). Reprend
  le Volet B de la 15-7b (AC 2 à 6, et la part « remise à zéro » des AC 7 à 12), ses tests 4 à 10b,
  12 et 13, avec l'historique des passes P2 à P4 de la 15-7b (Change Log de la fiche d'origine).
  **Passe de validation P4 de la 15-7b** (prompt versionné `15-7b-validate-prompt-p4.md` ; deux
  lentilles Opus en contexte frais, R regression hunter et F full-scope adversary ; rapports
  `target/gate-logs/15-7b-p4-{R,F}.md`, non versionnés). Bruts, recomptés depuis les rapports : R
  **3 MEDIUM, 6 LOW** ; F **4 MEDIUM, 7 LOW**. Après fusion (R4-1 = F-1, R4-5 = L-4, L-5 absorbé par
  F-2) : **0 CRITICAL, 0 HIGH, 6 MEDIUM, 11 LOW distincts**.

  | finding | sévérité | lentilles | objet | sort (fiche) |
  |---|---|---|---|---|
  | R4-1 = F-1 | MEDIUM | R, F | test 5 : référence insérée au montage ⇒ deux sociétés ⇒ 500 ; colonnes exclues non dites | référence construite après la remise à zéro dans une transaction annulée ; exclues `id`, `version`, `created_at`, `updated_at` (15-7b2) |
  | R4-2 | MEDIUM | R | la dérogation au découpage reposait sur un constat faux (3 des 4 MEDIUM de la P3 nés des correctifs de la P2) | **découpage** Volet A / Volet B — C-15-7-31 ; nouvelle dérogation de la 15-7b2 avec coupe prédéclarée et constat vérifié par versions |
  | R4-3 | MEDIUM | R | #43 citée comme ouverte : close, sans rapport | #43 retirée partout, #538 citée ; atomicité des quatre premières validations sans issue — signalée (15-7b1, 15-7b2) |
  | F-2 | MEDIUM | F | connexion rendue au pool avec `FOREIGN_KEY_CHECKS=0` si la future est abandonnée | `close_on_drop()` après chaque `acquire` ; `detach()` et `SET` d'erreur retirés ; test 8 sur `CONNECTION_ID()` ; `restore_tables_in_tx` → #540 — C-15-7-32 (15-7b2) |
  | F-3 | MEDIUM | F | aucune garde d'une clé d'une table préservée vers une table vidée | règle 3 du test 10b, mutation `invoices` (15-7b2) |
  | F-4 | MEDIUM | F | `reset_to_stub_in_tx` et le test 5 partagent la même énumération ouverte | test 5 : inventaire et comparaison sur `information_schema.COLUMNS`, liste fermée d'exclusions, montage perturbé (15-7b2) |
  | R4-4 | LOW | R | champs privés de `ResetOutcome` ; test 8 n'asserte rien | champs `pub`, assertions nommées (15-7b2) |
  | R4-5 = L-4 | LOW | R, F | test 12 : dépendances mockées inexactes, `children` absent | trois imports réels, `createRawSnippet` (15-7b2) |
  | R4-6 | LOW | R | test 13 : support et concurrence non dits | table de test `raw_sql`, `tokio::join!` (15-7b2) |
  | R4-7 | LOW | R | `bootstrap.rs:35-36`/`:31-34` décalés | `:36-37`, `:31-35` (15-7b2) |
  | R4-8 | LOW | R | choix applicables mal bornés ; C-15-7-24 sans renvoi | en-têtes bornés ; renvoi écrit dans les fiches et C-15-7-31 (le registre ne se réécrit pas) |
  | R4-9 | LOW | R | tolérance de 60 s du jeton omise | AC 4 et ligne `:1314` du manuel (15-7b2) |
  | L-1 | LOW | F | ordre des verrous sans `users` ni `api_keys` | ordre complet, AC 2 et AC 10 (15-7b2) |
  | L-2 | LOW | F | fichiers de documents non effacés, limite non écrite | inventaire § 5, « ne fait pas », manuel `:1314` (15-7b2) |
  | L-3 | LOW | F | `admin-manual.tex:691` hors du tableau | ligne ajoutée à l'AC 11 (15-7b2) |
  | L-6 | LOW | F | angle mort de la règle 2 non écrit | écrit à l'AC 3 (15-7b2) |
  | L-7 | LOW | F | bootstrap cas 1 : un stub par démarrage | hors périmètre, **signalé à l'orchestrateur** (15-7b2) |

  **Signal D5 constaté** — R4-2 (le constat de la dérogation était faux : vérifié par
  `git show 3846b206:` et `3c82e58f:` de la fiche, `retry_with` et `KEY_COLUMN_USAGE` absents à la
  borne P1, présents à la borne P2) et R4-1/F-1 (MEDIUM né de la remédiation P3, `9bbc52de`) — **et
  suivi** : la clause de la fiche s'applique, coupe Volet A / Volet B, décision de l'orchestrateur
  (C-15-7-31). La 15-7b2 reste au-dessus du seuil (huit modules) : nouvelle dérogation, avec coupe
  prédéclarée (story-zéro `kesh-db`). Propagation : grep des symptômes (`#43`, `detach`,
  `insérée par`, `au montage`, `35-36`, `31-34`, `KESH_JWT_EXPIRY_MINUTES`, `deux règles`,
  `users → api_keys`, `\$app`) sur les deux fiches, la fiche vidée, l'index et le registre. Recompte :
  **11 AC** (2 à 12), **8 tâches**, **13 tests** (4, 5, 6a-6d, 7, 8, 9, 10, 10b, 12, 13).
- 2026-10-08 — **Passe de validation P1** (prompt versionné `15-7b2-validate-prompt-p1.md` ; deux
  lentilles en contexte frais, R auditeur d'acceptation et F full-scope adversary ; rapports
  `target/gate-logs/15-7b2-p1-{R,F}.md`, non versionnés). Bruts, recomptés depuis les rapports : R
  **1 HIGH, 2 MEDIUM, 6 LOW** ; F **3 MEDIUM, 5 LOW**. Après fusion (R1 = F-1) : **0 CRITICAL, 1 HIGH,
  4 MEDIUM, 11 LOW distincts**.

  | finding | sévérité | lentilles | objet | sort |
  |---|---|---|---|---|
  | R1 = F-1 | HIGH | R, F | #542 désigne la 15-7b2, qui la renvoyait hors périmètre | **la 15-7b2 la ferme** : AC 13 (branche `(0, false)` gardée par `company_count == 0`), test 14, mutation, T3, Dev Notes réécrites, CHANGELOG (AC 12), `closes #542` ici et dans les deux fiches index — C-15-7-34 |
  | F-3 | MEDIUM | F | installations déjà touchées par #542 : la remise à zéro y rend 500 | étape 2 de l'AC 2 : sociétés provisoires sans utilisateur ni clé supprimées avant d'exiger une société (`MIN(id)` gardée si toutes le sont), `stub_companies_removed` (`ResetOutcome`, `details`), tests 6d (i)/(ii) et 6e, manuel `:1314` — C-15-7-34 |
  | F-2 | MEDIUM | F | gardes « sous le verrou de l'effacement » sans test ni mutation | test 7b déterministe (A tient `LOCK_SQL`, attente bornée sur `INNODB_TRX`, étape 7 commitée ⇒ `AlreadyFinalized`, tables intactes), mutation « lecture non verrouillante » — C-15-7-36 |
  | R2 | MEDIUM | R | forme de `RESET_CLEARED_TABLES` « dérivé » non spécifiable en `const` | `pub fn reset_cleared_tables() -> Vec<&'static str>`, nom repris partout — C-15-7-36 |
  | R3 | MEDIUM | R | test 4 vert quelle que soit la liste sur les tables non amorcées | relevé avant ; tables du montage nommées à `> 0` ; liste fermée des tables à 0 avant, assertée égale |
  | R4 | LOW | R | test 6c modifie un test de la 15-7a2 sans tâche | T6 et « Tests qui changent de sens » |
  | R5 | LOW | R | correspondance `SeedError` → `AppError` non écrite | AC 2 : `Db` ⇒ `AppError::Database`, `Sqlx` ⇒ `Internal`, 400/403 |
  | R6 | LOW | R | deux gardes de l'AC 2 sans test | test 7c (`None` ⇒ `Invariant`) ; garde `rows_affected` : angle mort déclaré (non provocable) |
  | R7 | LOW | R | erreur de rollback masquant un 1213 | AC 6 : journalisée, jamais substituée |
  | R8 | LOW | R | #540 n'est fermée qu'à moitié | `refs #540`, `refs #538` ; commentaire sur #540 par l'orchestrateur |
  | R9 | LOW | R | recompte à reporter | ci-dessous |
  | F-4 | LOW | F | motif `KESH.PRODUCTION.RESET` faux négatif sur le LaTeX | `KESH.{1,2}PRODUCTION.{1,2}RESET` ; `.env.example:272-278`, `docker-compose.dev.yml:28-29` ajoutés à l'AC 10 |
  | F-5 | LOW | F | doc de `get_company_for` (« FK RESTRICT ») faux | `helpers.rs:55-58` à l'AC 10, motif au grep |
  | F-6 | LOW | F | `serde_json` absent de `kesh-seed` | ajouté par la 15-7b1 (T1) ; T2 le vérifie |
  | F-7 | LOW | F | emplacement du test 10b ; littéral du stub dupliqué | `mod tests` de `backup.rs`, `"./test-schema"` ; constante substituée dans `test_fixtures.rs` (T1) |
  | F-8 | LOW | F | `backup.rs:33-86` | `:34-86` |

  **Signal D5 déclaré au Project Lead** : la sévérité de la P1 (1 HIGH) dépasse celle de la P4 de la
  15-7b (MEDIUM). **La coupe prédéclarée ne s'applique pas** : elle est réservée au **recyclage**
  (amendement D5), et aucun défaut de cette passe n'est né d'une remédiation — R1/F-1 et F-3 viennent du
  triage L-7 de la P4 (classé hors périmètre, non corrigé) et de l'existence de #542 ; F-2 de la
  conception C-15-7-11 ; R2 et R3 sont d'origine (vérifié par versions : `RESET_CLEARED_TABLES`
  « **dérivé** » et le test 4 « **chaque** table » figurent déjà dans la fiche 15-7b à sa naissance,
  commit `3846b206`, et inchangés aux commits `3c82e58f` et `9bbc52de`). Les défauts
  sont distincts de ceux de la P4. Le module `auth/bootstrap` était déjà compté : toujours **huit
  modules**. La remédiation change la conception (AC 13, étape 2 de l'AC 2) : **la validation n'est pas
  close**, P2 complète à lancer. Choix C-15-7-34, C-15-7-36. Propagation : grep de `542`,
  `RESET_CLEARED_TABLES`, `KESH.PRODUCTION.RESET`, `FK RESTRICT`, `33-86`, `multiplie les stubs`,
  `closes #540`, `ResetOutcome` sur les quatre fiches, les deux fiches index et le registre. Recompte :
  **12 AC** (2 à 13), **8 tâches**, **17 tests** (4, 5, 6a-6e, 7, 7b, 7c, 8, 9, 10, 10b, 12, 13, 14).
- 2026-10-08 — **Passe de validation P2** (prompt versionné `15-7b2-validate-prompt-p2.md` ; deux
  lentilles en contexte frais, R regression hunter et F full-scope adversary ; rapports
  `target/gate-logs/15-7b2-p2-{R,F}.md`, non versionnés). Bruts, recomptés depuis les rapports : R
  **3 MEDIUM, 3 LOW** ; F **2 MEDIUM, 5 LOW**. Après fusion (R2-1 = F-2 ; L-2 absorbé par R2-3 ;
  R2-4 = L-1) : **0 CRITICAL, 0 HIGH, 4 MEDIUM, 6 LOW distincts**.

  | finding | sévérité | lentilles | objet | sort |
  |---|---|---|---|---|
  | F-1 | MEDIUM | F | #528 : l'installation finalisée après une remise à zéro v0.12.x (une société, `users.company_id` mort) n'est réparée par aucun chemin | **réparation au démarrage** (AC 14) : `repair_orphan_principals` dans `ensure_admin_user`, une transaction, `attach_all_principals_in_tx` (helper unique de C-15-7-23), entrée `installation.principals_reattached` ; test 15 et trois mutations ; `closes #528` maintenu ; manuel `:1314` et CHANGELOG disent le vrai ; « plusieurs sociétés » laissé à la remise à zéro, écrit en limite — C-15-7-38 |
  | R2-1 = F-2 | MEDIUM | R, F | test 7b : `INNODB_TRX` exige `PROCESS` et n'est pas filtrable par base | `kesh_db::test_fixtures::attendre_une_requete_en_cours` (`PROCESSLIST` filtrée sur `DATABASE()`, sans privilège) ; la mutation « lecture non verrouillante » reste rouge (l'aide panique, la requête bloquée étant l'`UPDATE`) |
  | R2-2 | MEDIUM | R | cellule `:1314` sur trois lignes, phrase #542 détachant le « sauf » #528 | cellule d'une seule ligne ; la clause #528 suit « la session reste ouverte », la phrase #542 vient après elle |
  | R2-3 (+ L-2) | MEDIUM | R, F | AC 13 change la matrice de démarrage du manuel (`:1254`), absente de l'AC 11 | lignes `admin-manual.tex:1250-1259` (dont `:1255`, déjà inexacte) et `:967` à l'AC 11, T7 ; motifs `stub seule\|company provisoire\|placeholder` |
  | R2-4 = L-1 | LOW | R, F | verrous partagés de l'étape 2 absents de l'ordre « complet » | ordre conditionnel écrit à l'AC 2 et à la ligne du Pattern 5 (AC 10) |
  | R2-5 | LOW | R | « aucune donnée perdue » attribué à la règle 3 ; fait non gardé | attribution corrigée ; **règle 4** du test 10b (tables préservées portant une clé vers `companies` = {`users`, `api_keys`}) |
  | R2-6 | LOW | R | correspondance « complète » sans `StepAlreadyCompleted` ; doublon avec `AlreadyFinalized` | `AlreadyFinalized` retirée, `StepAlreadyCompleted` réemployée ; `ResetAttemptError`/`is_reset_retryable` deviennent l'extension de `SeedAttemptError`/`is_seed_retryable` posés par la 15-7b1 (un type, un prédicat, le test 13 pour les deux flux) — C-15-7-38 |
  | L-3 | LOW | F | sociétés superflues (#542) d'une installation de production : elles restent | limite écrite (Dev Notes, « ne fait pas », manuel `:1314`) |
  | L-4 | LOW | F | garde « ligne absente » inatteignable par HTTP ; la ligne recréée vaut étape 0 | angle mort écrit (AC 2, étape 1 ; « ne fait pas ») |
  | L-5 | LOW | F | commentaire `+layout.svelte:17-19` rendu inexact | AC 10 ; grep étendu à `frontend/src` |

  **Signal D5 déclaré au Project Lead** : trois des quatre MEDIUM sont **nés de la remédiation P1**
  (test 7b, cellule `:1314`, AC 13 sans ligne au manuel — vérifié par `git show 327ea9df^:` de la
  fiche : aucun des trois n'y figure) ; F-1 est d'origine (inventaire § 3). **La coupe prédéclarée ne se
  déclenche pas** : sa clause vise la sévérité **maximale de la passe précédente** (HIGH en P1), et ces
  MEDIUM sont des défauts de test et de manuel qu'une story-zéro `kesh-db` ne fermerait pas — formulation
  de la clause précisée dans la Dérogation (C-15-7-39). Le module `auth/bootstrap` était déjà compté :
  toujours **huit modules**. La remédiation ajoute une conception (AC 14) : **la validation n'est pas
  close**, P3 complète à lancer. Propagation : grep de `INNODB_TRX`, `AlreadyFinalized`,
  `ResetAttemptError`, `is_reset_retryable`, `stub seule`, `\b528\b`, `principals_reattached`,
  `y compris sur une installation déjà touchée`, `Une action neuve` sur les quatre fiches, l'index, le
  registre et `sprint-status.yaml`. Recompte : **13 AC** (2 à 14), **8 tâches**, **18 tests** (4, 5,
  6a-6e, 7, 7b, 7c, 8, 9, 10, 10b, 12, 13, 14, 15), **2 actions** d'audit neuves.
- 2026-10-08 — **Passe de validation P3** (prompt versionné `15-7b2-validate-prompt-p3.md` ; deux
  lentilles Sonnet en contexte frais, R regression hunter et F full-scope adversary ; rapports
  `target/gate-logs/15-7b2-p3-{R,F}.md`, non versionnés). Bruts, recomptés depuis les rapports : R
  **8 LOW** ; F **2 MEDIUM, 5 LOW**. Aucune fusion : **0 CRITICAL, 0 HIGH, 2 MEDIUM, 13 LOW distincts**.

  | finding | sévérité | lentilles | objet | sort |
  |---|---|---|---|---|
  | F3-1 | MEDIUM | F | la réparation au démarrage repointe en silence des clés d'API inertes vers la société vivante | **15-7b3** : clés orphelines **révoquées**, non repointées, ids au journal, manuel — C-15-7-41 |
  | F3-2 | MEDIUM | F | la restauration d'une sauvegarde rouvre #528 ; la réparation ne vit qu'au démarrage | **15-7b3** : même réparation dans la transaction de restauration, test — C-15-7-40 |
  | F3-3 | LOW | F | acteur de l'entrée du démarrage : un administrateur qui n'a rien fait | 15-7b3 : choix écrit, doc-comment et manuel — C-15-7-42 |
  | F3-4 | LOW | F | « toute route scopée répond 500 » inexact | AC 4 « Limite » et manuel `:1314` : 500 par `get_company_for`, listes vides ou erreur de clé étrangère ailleurs (et 15-7b3) |
  | F3-5 | LOW | F | test 7b : « tables intactes » vide sur un montage non peuplé | montage par `seed-demo` et une écriture, relevé avant, cinq tables à `> 0` |
  | F3-6 | LOW | F | « aucune insertion ne peut s'intercaler » non vérifié | écrit comme conjecture (AC 2 étape 4, Dev Notes), la sûreté n'en dépendant pas |
  | F3-7 | LOW | F | le manuel utilisateur ne dit pas l'effet de la remise à zéro | ligne `user-manual.tex:177-189` à l'AC 11, T7 |
  | R3-1 | LOW | R | test 13 : `ResetAttemptError`/`is_reset_retryable` survivants | `SeedAttemptError::Db` / `is_seed_retryable` |
  | R3-2 | LOW | R | « quatre variantes après la 15-7b1 » | « une fois cette fiche livrée — trois posées par la 15-7b1, `ResetForbidden` ici » |
  | R3-3 | LOW | R | variante du test 6e lue comme un cas réel | « montage défensif », renvoi aux cas laissés de la 15-7b3 |
  | R3-4 | LOW | R | index : `+layout.svelte:17-19` et la doc d'`ensure_admin_user` absents de l'attribution | lignes ajoutées au tableau de l'index (15-7b2 ; réparation : 15-7b3) |
  | R3-5 | LOW | R | `MULTI-TENANT-SCOPING-PATTERNS.md:313-314` revendiqué par deux fiches | `:313` ici (`reset`), `:314` à la 15-7b1 (`seed_demo`), index aligné |
  | R3-6 | LOW | R | `:1314` : marqueurs de revue dans le texte à écrire, « cellule » au lieu d'« item » | note à l'AC 11 et à T7 : `\item`, plusieurs phrases, renvois aux passes non recopiés |
  | R3-7 | LOW | R | `ensure_admin_user(test_config_no_env())` sans pool | 15-7b3, test 1 : `ensure_admin_user(&pool, &test_config_no_env())` |
  | R3-8 | LOW | R | la réparation au démarrage sans `retry_with` | 15-7b3 : angle mort assumé au démarrage (aucun trafic avant `bind`) ; à la restauration, route hors `retry_with`, import annulé et relancé — C-15-7-42 |

  **Signal D5 constaté, et suivi** : F3-1 et F3-2 sont **nés de la remédiation P2** (vérifié par versions
  de la fiche : `repair_orphan_principals|principals_reattached` → 0 occurrence à `327ea9df`, 10 occurrences
  sur 9 lignes à `ffcdcf8d` — unité corrigée par R4-4 de la P4), après une P2 à MEDIUM — la clause de coupe (C-15-7-39) est **déclenchée**. **Coupe selon la
  cause** (C-15-7-40), non selon l'axe prédéclaré : la réparation des installations déjà atteintes
  (AC 14, test 15, trois mutations, action `installation.principals_reattached`, lignes de manuel et de
  CHANGELOG) sort en **15-7b3**, qui `closes #528` ; cette fiche garde la prévention et passe à
  `refs #528`. Toujours **huit modules** (`auth/bootstrap` reste, pour #542). La 15-7b2 change de
  périmètre : **la validation n'est pas close**, P4 à lancer. Trend des passes de cette fiche : P1
  **1 HIGH, 4 MEDIUM** (Sonnet) → P2 **4 MEDIUM** (Opus) → P3 **2 MEDIUM** (Sonnet), tous nés de la
  remédiation précédente à partir de la P2 sauf F-1 de la P2. Propagation : grep de `ResetAttemptError`,
  `is_reset_retryable`, `313-314`, `principals_reattached`, `repair_orphan`, `AC 14`, `test 15`,
  `toute route scopée`, `deux actions`, `Deux actions`, `closes #528`, `aucune insertion ne peut` sur les
  fiches 15-7a1, 15-7a2, 15-7b1, 15-7b2, 15-7b3, la fiche 15-7b, l'index, le registre et
  `sprint-status.yaml`. Recompte : **12 AC** (2 à 13 ; l'AC 14 renvoie à la 15-7b3), **8 tâches**,
  **17 tests** (4, 5, 6a-6e, 7, 7b, 7c, 8, 9, 10, 10b, 12, 13, 14), **1 action** d'audit neuve.
- 2026-10-08 — **Passe de validation P4** (prompt versionné `15-7b2-validate-prompt-p4.md` ; deux
  lentilles **Opus** en contexte frais, R regression hunter et F full-scope adversary ; rapports
  `target/gate-logs/15-7b2-p4-{R,F}.md`, non versionnés), remédiée avec la P1 de la 15-7b3 (décisions de
  l'orchestrateur). Bruts, recomptés depuis les rapports : R **2 MEDIUM, 3 LOW** ; F **3 MEDIUM, 4 LOW**.
  Après fusion (R4-1 = F4-1, R4-2 = F4-2) : **0 CRITICAL, 0 HIGH, 3 MEDIUM, 7 LOW distincts**.

  | finding | sévérité | lentilles | objet | sort |
  |---|---|---|---|---|
  | R4-1 = F4-1 | MEDIUM | R, F | C-15-7-41 (clé orpheline révoquée, jamais repointée) appliquée au seul démarrage : la remise à zéro et `language` repointent — réveillent — les clés orphelines | **une règle, une fonction** : `reattach_orphan_principals_in_tx` (AC 4) révoque toute clé active orpheline **puis** la repointe vers la société s'il y en a une ; appelée par `reset_demo`, la branche « aucune société » de `language`, et par la 15-7b3 au démarrage et à la restauration ; `ResetOutcome.principals`, `details.api_keys_revoked` ; tests 6a-6c complétés, test 6f, mutations — C-15-7-45 (révise C-15-7-41 : révoquer **et** repointer) |
  | R4-2 = F4-2 | MEDIUM | R, F | le manuel renvoie au bouton de réinitialisation, refusé (403) même à l'administrateur sans `KESH_PRODUCTION_RESET` | lignes `user-manual.tex:177-189`, `admin-manual.tex:1314` et `:691` disent la marche (poser le drapeau, redémarrer, réinitialiser, le retirer) et l'impasse comme choix de sécurité existant ; renvoi de la 15-7b1 corrigé ; Dev Notes ; #534 — C-15-7-48 |
  | F4-3 | MEDIUM | F | la réparation de #542 par la remise à zéro n'est atteignable par aucun écran | **passe à la 15-7b3**, au démarrage (ex-test 6e → test 4 de la 15-7b3) ; ici, la prévention seule (AC 13), `refs #542` ; étape 2 de l'AC 2 réduite (plus d'une société ⇒ `Invariant`), test 6d (ii) ; manuel, CHANGELOG, Dev Notes — C-15-7-46 (révise C-15-7-34) |
  | R4-3 | LOW | R | « trois variantes posées par la 15-7b1 » faux | « deux d'origine (`Db`, `Sqlx`), `StepAlreadyCompleted` (15-7b1), `ResetForbidden` ici » |
  | R4-4 | LOW | R | « 9 occurrences » : 9 lignes, 10 occurrences | Dérogation et Change Log P3 : « 10 occurrences sur 9 lignes » (le registre, C-15-7-40, ne se réécrit pas : renvoi en C-15-7-49) |
  | R4-5 | LOW | R | `:1314` : le « sauf » ne couvre pas l'installation touchée à une société | « quel que soit son nombre de sociétés » |
  | L4-1 | LOW | F | tableau `admin-manual.tex:683-693` rogné dans le PDF | défaut et contrôle écrits à l'AC 11 (ligne `:691`) et à T7 |
  | L4-2 | LOW | F | « CR à ouvrir » : #534 existe | `refs #534` (Issues, Dev Notes, References) |
  | L4-3 | LOW | F | test 4 : `api_keys` inchangées vrai à vide | clé saine et jeton de réinitialisation au montage ; la clé saine non révoquée |
  | L4-4 | LOW | F | test 8 : premier déclencheur du dépôt, pré-requis non dits | `raw_sql`, `SIGNAL SQLSTATE '45000'`, `@@log_bin`, privilèges |

  **Signal D5 déclaré au Project Lead — clause de coupe déclarée, non appliquée** (décision de
  l'orchestrateur, C-15-7-45, C-15-7-48) : R4-1/F4-1 (MEDIUM) est un **résidu de propagation** de la
  décision C-15-7-41 de la P3 — le symptôme « repointer une clé orpheline » corrigé à un site sur trois
  (vérifié par versions : `api_keys_repointed` présent à `327ea9df`, `ffcdcf8d` et `a8302032` ; la
  contradiction naît à `a8302032`) —, non un défaut de conception ; R4-2/F4-2 (MEDIUM) est né de la
  remédiation P3 (ligne `user-manual.tex:177-189`, F3-7 : 0 occurrence de `177-189` à `ffcdcf8d`, 3 à
  `a8302032`). Les deux suivent une P3 à MEDIUM : la clause (C-15-7-39) est formellement atteinte. **Pas
  de coupe** : l'axe prédéclaré (story-zéro `kesh-db`) ne fermerait ni un défaut de propagation entre
  fiches sœurs ni une ligne de manuel (motif de C-15-7-39 et C-15-7-40) ; F4-3 est d'origine (C-15-7-34,
  P1). Toujours **huit modules** (la règle des principaux vit dans `companies`, déjà compté ; la
  suppression des sociétés superflues quitte `kesh-seed` sans retirer de module). Trend des passes de
  cette fiche : P1 **1 HIGH, 4 MEDIUM** (Sonnet) → P2 **4 MEDIUM** (Opus) → P3 **2 MEDIUM** (Sonnet) →
  P4 **3 MEDIUM** (Opus ×2). La remédiation change la conception (AC 2, AC 4) : **la validation n'est pas
  close**, P5 à lancer. Propagation : grep de `attach_all_principals`, `attach_users_in_tx`,
  `api_keys_repointed`, `stub_companies_removed`, `superflu`, `réparée par la réinitialisation`,
  `AC 2.2`, `closes #542`, `CR à ouvrir`, `177-189`, `179-189`, `Réinitialiser pour la production`,
  `trois posées`, `\b9\b` (décomptes), `272-278` sur les fiches 15-7*, l'index, la fiche 15-7b, le
  registre et `sprint-status.yaml`. Recompte : **12 AC** (2 à 13 ; l'AC 14 renvoie à la 15-7b3),
  **8 tâches**, **17 tests** (4, 5, 6a, 6b, 6c, 6d, 6f, 7, 7b, 7c, 8, 9, 10, 10b, 12, 13, 14 ; le 6e
  renvoie à la 15-7b3), **1 action** d'audit neuve.
- 2026-10-08 — **Passe de validation P5** (prompt versionné `15-7b2-validate-prompt-p5.md` ; deux
  lentilles **Sonnet** en contexte frais, R regression hunter et F full-scope adversary ; rapports
  `target/gate-logs/15-7b2-p5-{R,F}.md`, non versionnés). Bruts, recomptés depuis les rapports : R
  **0 MEDIUM, 5 LOW** ; F **1 MEDIUM, 3 LOW**. Aucune fusion : **0 CRITICAL, 0 HIGH, 1 MEDIUM, 8 LOW
  distincts**. Remédiation **sur la spec seule** (décisions de l'orchestrateur).

  | finding | sévérité | lentilles | objet | sort |
  |---|---|---|---|---|
  | F5-1 | MEDIUM | F | la recette de sortie de la démonstration (C-15-7-48) est inexécutable : `docker-compose.yml` et `docker-compose.prod.yml` ne transmettent pas `KESH_PRODUCTION_RESET` (liste `environment:` explicite, sans `env_file`) | défaut plus large (SMTP et une vingtaine de variables) : **#550** et story **15-11-configuration-transmise** (orchestrateur) ; ici, **dépendance déclarée** — la 15-11 est mergée **avant** celle-ci (en-tête, T8 : contrôle `grep` et `docker compose config` avant merge), `refs #550`, grep de l'AC 10 étendu aux deux compose de production, manuel (`:691`, `:1314`) et Dev Notes ; la fiche ne modifie pas les compose — C-15-7-51 (révise C-15-7-48) |
  | R5-1 | LOW | R | la coupe prédéclarée n'embarque pas le test 6f | « tests 5 (part colonnes), 6f, 10 et 10b » |
  | R5-2 | LOW | R | `routes/onboarding.rs:225-231` et `errors.rs:282` contredisent C-15-7-48, sans attribution | attribués à l'AC 10 et à l'index |
  | R5-3 | LOW | R | le débordement ne touche pas que le tableau `:683-693` | AC 11 (`:691`) et T7 : les **dix** tableaux de `sec:env-vars` (huit rognés, deux décalés, recensés au `.log` et au PDF aplati), un même geste dans la section seule — C-15-7-52 |
  | R5-4 | LOW | R | « même forme que `installation.repaired` » faux de l'entrée entière | les champs `api_keys_*` seuls ; `user_ids` en plus dans `installation.repaired` — C-15-7-52 |
  | R5-5 | LOW | R | test 8 : conduite non fixée si `@@log_bin` est actif | `assert_eq!(log_bin, 0, …)` nommant le pré-requis, jamais de sortie anticipée |
  | F5-2 | LOW | F | numéros de ligne des `.tex` sans réserve de relocalisation | « Renvois de ligne » étendu aux manuels : relocalisation par le texte après rebase ; T7 |
  | F5-3 | LOW | F | le manuel ne dit pas qu'une clé saine garde son scope | `:1314` : « une clé d'API saine garde son droit d'écriture si elle l'avait » |
  | F5-4 | LOW | F | test 8 : `foreign_key_checks = 1` vrai par construction | `CONNECTION_ID()` est la preuve, `foreign_key_checks` une ceinture (test et mutation) |

  **Signal D5** : F5-1 porte sur la recette écrite par la remédiation P4 (C-15-7-48), après une P4 à
  MEDIUM — la clause de coupe (C-15-7-39) est formellement atteinte. **Pas de coupe** : la cause est
  hors de la fiche (les compose de production, antérieurs à l'epic) et se ferme par la story 15-11 ;
  l'axe prédéclaré (story-zéro `kesh-db`) ne la fermerait pas (motif de C-15-7-39, C-15-7-40). Signal
  déclaré au Project Lead ici. Toujours **huit modules** (les compose ne sont pas touchés).
  Trend des passes de cette fiche : P1 **1 HIGH, 4 MEDIUM** (Sonnet) → P2 **4 MEDIUM** (Opus) → P3
  **2 MEDIUM** (Sonnet) → P4 **3 MEDIUM** (Opus ×2) → P5 **1 MEDIUM, 8 LOW** (Sonnet ×2) ; le MEDIUM de la
  P5 est traité par une dépendance déclarée, les LOW appliqués : **0 au-dessus de LOW — validation
  close**, la remédiation P5 ne touchant que la spec (fiche, index, registre, `sprint-status.yaml`).
  Propagation : grep de `10 et 10b`, `683-693`, `même forme que`, `must remain unset`, `225-231`,
  `errors.rs:282`, `docker-compose`, `PRODUCTION_RESET`, `redémarrer Kesh` sur les fiches 15-7*, l'index
  et le registre — résidus : l'historique (Change Log P4, `683-693`) seul ; la fiche 15-7b1 (renvoi
  `user-manual.tex:179-189`) ne mentionne ni `.env` ni les compose et reste vraie ; la fiche 15-7b3 n'est
  pas touchée. Recompte : **12 AC** (2 à 13 ; l'AC 14 renvoie à la 15-7b3), **8 tâches**, **17 tests**
  (4, 5, 6a, 6b, 6c, 6d, 6f, 7, 7b, 7c, 8, 9, 10, 10b, 12, 13, 14 ; le 6e renvoie à la 15-7b3),
  **8 modules**, **1 action** d'audit neuve.
- **2026-10-08 — Renvois mis à jour par l'orchestrateur** : la 15-11 a été découpée (choix C77) en 15-11a
  (compose et documentation, `closes #550`) et 15-11b (lecture unique des variables). Cette fiche ne dépend que
  de la **15-11a** : les douze renvois actifs (lignes 29, 42, 51, 475, 540, 541, 558, 604, 686, 689, 776, 784) disent désormais
  « 15-11a ». Les mentions de « 15-11 » dans le Change Log P5 et dans C-15-7-51 restent historiques. Ordre de
  merge : **15-11a, puis 15-7b2**. Au développement, la recette de sortie de la démonstration doit dire
  `docker compose up -d` (et non « redémarrer » : `restart` ne relit pas `.env`).
- **2026-10-08 — Ajustement après la remédiation P2 de la 15-11a (C81)** : la 15-11a (AC12 (j)) reprend la mise en page des dix tableaux de `sec:env-vars`, que cette fiche avait prévue (AC 11 ligne `:691`, T7, C-15-7-52) ; elle merge avant cette story. AC 11 et T7 ne portent plus que le contrôle, après rebase, de la lecture entière de la cellule `KESH\_PRODUCTION\_RESET` au PDF aplati (renvoi à la 15-11a, AC12 (j)). Le motif de contrôle `KESH.{1,2}PRODUCTION.{1,2}RESET` (AC 11) et le T8 énumèrent désormais les mentions que la 15-11a ajoute au manuel et au CHANGELOG, et rappellent que la 15-11a interdit de nommer la variable dans un commentaire des compose. Choix **C-15-7-54** (révise C-15-7-52). Recompte : **12 AC**, **8 tâches**, **17 tests** — inchangés (la mise en page était une sous-clause de l'AC 11 et de T7, non un décompté à part). Propagation : grep de `sec:env-vars`, `\paragraph`, `Overfull`, `C-15-7-52` sur la fiche et l'index 15-7 — résidus : l'historique (Change Log P4 et P5, R5-3, L4-1, C-15-7-52) seul ; la fiche 15-7b3 (lignes 40 et 435) mentionne `sec:env-vars` et C-15-7-52 pour son propre texte, renvoi traité à part (signalé à l'orchestrateur).
- **2026-10-08 — Recette de redémarrage (C-15-7-55)** : la recette de sortie de la démonstration (`admin-manual.tex:1314`, cellule `:691`, T7) disait « redémarrer » ; elle prescrit désormais `docker compose up -d` après la pose puis après le retrait de `KESH_PRODUCTION_RESET`, car `restart` ne relit pas `.env` (référence : 15-11a, AC12 f, C83, C84, validée et close). `user-manual.tex:177-189` (ligne de la fiche) ne contient aucune consigne de redémarrage : inchangé. Grep des symptômes (`redémarr`, `restart`) sur les fiches 15-7* : les autres occurrences sont historiques (Change Log) ou décrivent le comportement du serveur (redémarrages avant `/setup`), non une recette d'exploitant.
- **2026-10-09 — Développement** (Claude Opus 5.5, worktree `kesh-15-7b2`, base `origin/main` `181efa3c`, inchangée en fin de développement). T0 écrit avant tout code (11 écarts, dont registre à 114 routes et manuel déjà périmé, api-external, liste « Deny list » disparue). T1 à T8 livrés : `reset_demo` en une transaction par essai, rejouée, connexion fermée à sa libération, gardes sous le verrou de l'effacement ; société conservée en place ; règle unique des principaux orphelins ; vidage dérivé de la liste canonique gardé par quatre règles sur le schéma ; bootstrap gardé contre #542 ; `installation.reset` dans les quatre catalogues ; registre 108 / 4 / 2. Gates avant rebase (`d64a9545`) : backend 3180 / 3180, Vitest 1161 / 1161, E2E 246 / 8 attendus (7 KF-029 + #424 vert seul) ; **rebasée sur `056997b0` (15-1a-ii)**, gates de référence sur l'état rebasé : backend 3197 / 3197, Vitest 1164 / 1164, E2E 247 / 7 KF-029 ; 31 mutations, 31 rouges (mesurées avant rebase, sur le même code de la story). Choix C-15-7b2-1 à 4. Recompte : **12 AC**, **8 tâches**, **17 tests** de la fiche écrits (4, 5, 6a, 6b, 6c, 6d en deux, 6f en trois, 7, 7b, 7c, 8, 9, 10, 10b, 12 en trois, 13, 14) **plus** le 13b (C-15-7b2-2). Statut : **review**.
- **2026-10-09 — Revue de code P1** (prompt versionné `15-7b2-review-prompt-p1.md`, commit `e3368ba7` ; trois lentilles **Sonnet** en contexte frais, B Blind Hunter, E Edge Case Hunter, A Acceptance Auditor ; rapports `kesh-gate-logs/15-7b2-review-p1-{B,E,A}.md`). Bruts, recomptés depuis les rapports : B **0 MEDIUM, 6 LOW** ; E **0 MEDIUM, 7 LOW** ; A **1 MEDIUM, 3 LOW**. Après fusion (A-1 = E-3 ; B-6 = E-6) : **0 CRITICAL, 0 HIGH, 1 MEDIUM, 14 LOW distincts**. Remédiation (C-15-7b2-5, C-15-7b2-6) : A-1 = E-3 corrigé au manuel d'administration (PDF régénéré, contrôlé aplati), au CHANGELOG et à l'AC 11 ; A-2, A-3, A-4 appliqués (sorties collées) ; les LOW de code écrits en dette avec leur motif (B-1 à B-6, E-1, E-2, E-4 à E-7). **La remédiation ne touche aucune ligne de code ni de test** : le dernier commit de code reste `5fe1f918`, et ses gates sur l'état rebasé (backend 3197 / 3197, Vitest 1164 / 1164, E2E 247 / 7 KF-029) restent valables. Trend : P1 **1 MEDIUM** (d'origine de fiche, non né d'une remédiation) → corrigé sans code. **Boucle close** (règle « passe ciblée » : une remédiation qui ne touche aucune ligne de code de production ne rouvre pas la boucle). Statut : **done**.

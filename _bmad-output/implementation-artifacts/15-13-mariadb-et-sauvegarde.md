# Story 15.13 : MariaDB et sauvegarde avant import — fiche index, découpée en 15-13a et 15-13b

Status: split

<!-- Découpée le 2026-10-09 après la validation P3 (signal D5 de recyclage levé deux passes de suite),
     décision de l'orchestrateur — choix C-15-13-16 (`epic-15-choix-autonomes.md`). Corps de la fiche vidé :
     la **version complète avant découpage** est au commit **8a9bcd27**
     (`git show 8a9bcd27:_bmad-output/implementation-artifacts/15-13-mariadb-et-sauvegarde.md`). Le Change
     Log de la spécification et des validations P1 et P2 est conservé ci-dessous ; la remédiation P3 est
     appliquée dans les deux fiches filles. -->

**Issues** : #551 (fermée par la **15-13a**), #552 et #576 (fermées par la **15-13b**) ; `refs #558`
(15-13b), `refs #577` (15-13a) ; #575 et #554 citées, hors périmètre.

## Découpage

**La couture** : #551 (MariaDB : port, mots de passe, avertissement, indice 1045 — tout ce qui touche la
base et son accès) d'un côté ; #552 et #576 (la sauvegarde pré-import : son dossier, son mode, le message de
son échec) de l'autre. #576 ne se sépare pas de #552 : elle naît du nouveau défaut `/data/backup`, qui fait
de l'échec d'écriture le cas nominal hors Docker (C-15-13-9). Aucune des deux fiches n'appelle ce que
l'autre ajoute ; elles touchent les **mêmes fichiers** (`docker-compose.yml`, `.env.example`, `config.rs`,
`configuration_transmise.rs`, `admin-manual.tex` § *Passer à la 0.13.0*, `DOCKER_START.md`,
`CHANGELOG.md` `[0.13.0]`) à des endroits distincts.

| fiche | contenu | issues | modules de code | dépend de |
|---|---|---|---|---|
| **15-13a** — `15-13a-mariadb-non-publiee-mots-de-passe-obligatoires.md` | port de MariaDB non publié ; `${MARIADB_*_PASSWORD:?…}` (refus nommé de Compose, vérifié en CI dans les deux sens) ; `.env.example` sans `kesh_dev` ; avertissement au démarrage (mot de passe publié **ou gabarit du manuel**, F-P3-3) ; indice sur le 1045 ; `configuration_transmise.rs` (service `mariadb`, scission de l'interpolation, `VALEURS_COMPOSEES`) ; manuel (ports, étape `.env`, hex, paragraphe MariaDB de la mise à jour, changement de mot de passe, `exec mariadb`, dépannage), brochure, `DOCKER_START.md`, retrait d'`init-demo.sh`, `docs/ci.md` ; CHANGELOG **Sécurité** #551 et **Retiré** | **closes #551**, refs #577 | `config`, `main` (2) | 15-11a, 15-11b (mergées) |
| **15-13b** — `15-13b-sauvegarde-avant-import-persistante.md` | défaut `/data/backup` (`config::env_nonempty` inchangé), montage fixe `./backup` des deux compose, fichier `0600` sans écrasement, dossier `0700` ; variante `AdminPreImportBackupFailed`, `avant_sauvegarde` et le tri des 17 sites `AdminFullImportFailed` de `routes/admin.rs` ; quatre locales ; recettes hors Docker (AC 16, élargi aux modes `cargo run` documentés) ; `.gitignore` (motifs **ancrés**, R3-1) et `.dockerignore` ; rapatriement pour restaurer ; manuel (sauvegarde) ; CHANGELOG **Corrigé** #552, #576 et **Sécurité** (dossiers ignorés) | **closes #552**, **closes #576**, refs #558 | `config`, `routes/admin`, `errors`, catalogues `kesh-i18n` (4) | 15-11a, 15-11b (mergées) |

**Ordre suggéré** : 15-13a, puis 15-13b — #551 est P2 (sécurité), #552 et #576 P3. L'ordre inverse est
possible : la seule conséquence est le **recompte des gestes** du paragraphe « Pour qui garde son fichier
compose » (`admin-manual.tex:1793`) et de `CHANGELOG.md:46`, que fait la fiche mergée en second. Nombres,
par fichier (`docker-compose.yml` / `docker-compose.prod.yml`) : aujourd'hui 2 / 2 ; 15-13a seule 4 / 2 ;
15-13b seule 3 / 3 ; les deux 5 / 3.

**Si une seule est mergée au tag v0.13.0** — chaque fiche ne réécrit que ce que **son** changement rend
faux, si bien que l'autre partie du CHANGELOG et du manuel reste vraie :
- **15-13a seule** : CHANGELOG `[0.13.0]` avec l'entrée **Sécurité** #551, la rubrique **Retiré**
  (`init-demo.sh`), quatre phrases de `:46` réécrites et `:52` corrigée ; la phrase « la sauvegarde
  pré-import reste dans `/tmp` » reste (vraie). Manuel : parties MariaDB ; les « défaut `/tmp` »
  (`:757`, `:1664`, `:1699`) restent (vrais).
- **15-13b seule** : CHANGELOG avec les entrées **Corrigé** #552 et #576, la **Sécurité** « dossiers
  montés ignorés », la phrase `/tmp` et le décompte des gestes de `:46` réécrits ; les trois autres phrases
  de `:46` restent (vraies tant que les mots de passe ont un défaut). Manuel : parties sauvegarde. #551
  reste ouverte, et la release ne peut pas se dire « MariaDB fermée ».

### Recompte aux deux bornes

Borne basse : la fiche unique au commit **8a9bcd27** (comptes vérifiés par la lentille R de la P3).
Borne haute : les deux fiches filles de ce commit, recomptées par `grep` (`'^[0-9]+\. \*\*'` pour les AC,
`'^- \[ \] \*\*T'` pour les tâches, `grep -oE '\*\*M[0-9]+\*\*' | sort -u` pour les mutations, `'^| '` pour
les tableaux).

| Objet | Fiche unique (8a9bcd27) | 15-13a | 15-13b | Écart expliqué |
|---|---|---|---|---|
| AC (numéros) | 16 | 11 (1–6, 10–14) | 10 (7–16) | 5 numéros partagés (10–14), sous-points répartis : 11 + 10 − 5 = 16, aucun perdu |
| Tâches | 11 (T0–T10) | 11 | 9 (sans T4, T7) | T4 (CI) et T7 (recette MariaDB) propres à la 15-13a ; les autres scindées |
| Lignes de test | 19 (18 Rust + 1 CI) | 9 (8 + CI) | 11 | 9 + 11 = 20 : **+1**, la ligne **3a** (`valeurs`, ci-dessous) |
| Fonctions de test Rust | 19 (15 neuves, 4 complétées) | 8 (7 neuves, 1 complétée) | 12 (8 neuves, 4 complétées) | 20 = 19 **+ `valeurs`**, modifiée par le resserrement de `VALEURS_COMPOSEES` mais jamais comptée (constat du remédiateur : M5 rougit `valeurs`, non `transmission` ni le test 1) |
| Mutations | 40 (M1–M40) | 22 | 20 | 42 = 40 + **M41** (R3-1, 15-13b) + **M42** (F-P3-3, 15-13a) ; aucune en double (`sort -u` sur l'union → 42, de M1 à M42 sans trou) |
| Tableau des cas | 11 lignes | 10 | 3 | 13 = 11 + 2 lignes scindées (« compose gardé », « image ancienne ») |
| Inventaire | 57 lignes | 47 | 36 | totaux recomptés aux deux bornes (`grep -c '^| '` sur chaque tableau, en-tête exclu) ; la ventilation ligne à ligne écrite ici au découpage (« 29 passées entières… → 36 + 28 ») **ne se refaisait pas** (R4-7 de la validation P4 de la 15-13a : 29 + 6 ≠ 36) et est retirée. Depuis la validation P4 : 48 (15-13a) et 41 (15-13b), détail au Change Log de chaque fille |
| Angles morts | 12 | 4 | 8 | aucun perdu ; celui du gabarit réécrit (root seul, tracé par issue) |
| Modules de code | 5 | 2 | 4 | `config` dans les deux ; union = 5 |

## Change Log

- 2026-10-09 — Spécification (agent de spécification, Opus 5.5, en autonomie). 14 AC, 11 tâches (T0–T10),
  13 lignes de test (13 fonctions Rust dont 10 neuves, 1 étape CI), 27 mutations ; choix C-15-13-1 à C-15-13-7. Issues à ouvrir signalées à
  l'orchestrateur : section Synology du manuel (pré-script et aperçu sans service `mariadb`, nom de volume
  faux) ; message `error-admin-full-import-failed` faux quand l'écriture de la sauvegarde échoue.
  *(Ouvertes depuis : **#575** et **#576**.)*
- 2026-10-09 — **Remédiation de la validation P1** (agent remédiateur, Opus 5.5, en autonomie). Passe P1 :
  deux lentilles Opus 5.5 en contexte frais — R (auditeur d'acceptation) **0 CRITICAL / 0 HIGH / 4 MEDIUM /
  12 LOW**, F (adversaire plein périmètre) **0 / 0 / 7 MEDIUM / 8 LOW** ; doublons R-1 = F1, R-3 = F6,
  R-2 ≈ F4 — soit **8 MEDIUM distincts** ; 20 LOW, dont 17 distincts (L1 = F8, L2 = F9, L4 = F10). Tous remédiés :
  - **#576 absorbée** (F7, décision de l'orchestrateur, C-15-13-9) : AC 15 (variante
    `AdminPreImportBackupFailed`, `avant_sauvegarde` à l'appel, quatre locales et repli Rust) et AC 16
    (recettes `cargo run` du dépôt — `CLAUDE.md`, une ligne ; `docs/testing.md`) ; la PR porte
    `closes #576`.
  - **Sauvegarde `0600` gardée, rapatriement au manuel** (R-2/F4, décision de l'orchestrateur, C-15-13-8) :
    AC 9, 11 e, T0, ligne « restaurer après un import raté », angle mort ACL Synology.
  - **Contrôle de fin de mise à jour par connexion à la base** (F3, décision de l'orchestrateur,
    C-15-13-10) : AC 11 f, T7 ; lignes « root neuf sans `ALTER USER` » et « gabarit littéral » (R-4) au
    tableau des cas ; « aucun cas ne casse en silence » réécrit.
  - **YAML du message `:?`** (R-1/F1) et **scission `:?`/`?`** (F2, M28-M29) : AC 2 c, 10 a, 10 d,
    C-15-13-12.
  - **Inventaire recompté** (R-3/F6) : grep du T9 rejoué au `HEAD` (83 fichiers, chacun trié), plus
    `MARIADB_` et `cargo run -p kesh-api` ; `init-demo.sh` résolu (AC 12 b, C-15-13-11) ; numéros de
    ligne corrigés (`docker-compose.dev.yml:15`, `:54`, `:56-59` ; `ci.yml:55` ; `release.yml:26` ;
    `admin-manual.tex:1761-1776`, `:1746`).
  - **Propagation** (F5) : AC 13 c étendu aux quatre phrases de `CHANGELOG.md:46` ; AC 11 f à l'encadré
    `:1761-1776` (refus de Compose affiché au terminal).
  - LOW : `percent-encoding` (L1/F8) ; deux fonctions pour l'indice 1045 et repli `(using password`
    (L2/F9) ; `write_backup_file` nommée (L3) ; hôte du test 8 mesuré au T0 (L4/F10) ; contrôles PDF
    rendus discriminants, vérifiés rouges sur le PDF aplati d'avant (L5) ; #575/#576 cités (L7) ; recettes
    de dev (L8, avec F7) ; nom de volume `<projet>_kesh-mariadb-data` (L9) ; limite root au CHANGELOG
    (L10) ; `VALEURS_COMPOSEES` resserré (L11) ; textes « trois » (L12) ; fichier partiel supprimé (F11) ;
    commentaire `admin.rs:475-477` (F12) ; Compose nomme une variable à la fois (F13) ; ordre existant →
    neuf (F14) ; jetons fantômes au manuel et pré-script Synology à joindre à #575 (F15) ; `cd` du script
    de sauvegarde (AC 11 h).
  - **Recompte** (depuis la fiche) : **16 AC** (14 → 16), **11 tâches** (T0–T10), **18 lignes de test**
    (13 → 18) portant **18 fonctions Rust** (14 neuves, 4 existantes complétées) dans un fichier neuf et cinq
    existants, plus 1 étape CI ; **37 mutations** (M1–M37, 27 → 37). **Modules** : 5 (`config`, `main`,
    `routes/admin`, `errors`, catalogues `kesh-i18n`) — seuil de découpage atteint, non franchi.
  - **Signal de découpage** (D5) : la passe P1 est la première, aucune comparaison de sévérité n'est
    possible ; le décompte de modules passe de 3 à 5 par l'absorption de #576 — déclaré au Project Lead
    via l'orchestrateur.
- 2026-10-09 — **Remédiation de la validation P2** (agent remédiateur, Opus 5.5, en autonomie). Passe P2 :
  deux lentilles **Sonnet** en contexte frais — R (regression hunter) **0 CRITICAL / 0 HIGH / 3 MEDIUM /
  7 LOW**, F (adversaire plein périmètre) **0 / 0 / 4 MEDIUM / 8 LOW** ; doublon R2-3 = F4 — soit **6 MEDIUM
  distincts** ; 15 LOW, dont 14 distincts (le cas `umask 077` de R2-10 = F12). **Trend** : P1 (Opus 5.5)
  8 MEDIUM distincts → P2 (Sonnet) 6 MEDIUM distincts ; 0 CRITICAL/HIGH aux deux passes. Tous remédiés :
  - **`init-demo.sh` retiré** (F2, décision de l'orchestrateur, C-15-13-13, qui remplace C-15-13-11) :
    redondant avec la démonstration semée par l'application (`seed-demo` → `kesh_seed`), appelé par
    personne (`git grep`) ; AC 12 b réécrit, T6/T7, CHANGELOG « Retiré », inventaire.
  - **Test 16** (R2-1) : l'interdit est la **parenthèse entière** de l'ancienne promesse, par locale, avec
    assertion de montage (l'interdit figure dans l'ancien texte de sa locale) ; vérifié par `grep -cF`
    qu'aucune négation naturelle de l'AC 15 c ne le contient ; AC 15 c dit que la négation est la forme
    attendue ; mutation M38.
  - **Registres réalignés** (R2-2) : `sprint-status.yaml` (lignes 1 et 359) et la fiche d'epic (`:29-35`)
    — 16 AC, 19 fonctions, 40 mutations, 5 modules, C-15-13-1 à 15, `closes #551 #552 #576`, #575/#576.
  - **Brochure** (R2-3/F4) : AC 11 m, T5, inventaire ; T9 gagne un grep **par la valeur** du geste de
    démarrage (`docker[ -]compose( -f …)? up`) sur `docs`, `website`, `README.md`, `DOCKER_START.md` —
    sites triés à l'inventaire (`website/` : aucune commande sur `docker-compose.yml`).
  - **Dossiers montés hors du dépôt** (F1, C-15-13-14) : AC 8 e, propagé par le symptôme à `./inbox` et
    `./documents` ; test 19 dérivé de `MONTAGES`, mutations M39-M40.
  - **`DOCKER_START.md:134-137`** (F3) : AC 12 a-bis — diagnostic du 1045 d'abord, `down -v` réservé à une
    installation sans données.
  - LOW : numéros de ligne (`admin-manual.tex:1667-1672`, `errors.rs:448-452`, `admin.rs:467-470` ajouté,
    C-15-13-5 rectifié `:573` par ajout) (R2-4, R2-6) ; AC 16 c complété (`password-recovery.spec.ts:16`,
    `global-setup.ts:34`) (R2-5) ; C-15-13-7 renvoie à C-15-13-9 par ajout (R2-7) ; manuel par **fragment**
    `:?`, pas de ligne `DATABASE_URL` entière (76 caractères) (R2-8) ; contrôle des placeholders précédé de
    `config -q && echo 'compose lisible'` (R2-9) ; fichier partiel nommé par le `warn!`, angle mort (R2-10) ;
    `SELECT CURRENT_USER()`, compte relevé pour `127.0.0.1` et le nom du service, commandes coupées par `\`
    (F5) ; `docs/ci.md` « 4 jobs » → 3, le job `e2e` fantôme laissé à une issue (F6, C-15-13-15) ; helper
    `lancer_binaire` dans `tests/common/binaire.rs` (F7) ; refus de Compose sur toute sous-commande, mesuré
    au T0 (F8) ; portée exacte de l'AC 16 et date de la recette du `CLAUDE.md` (F9) ; motifs PDF à
    apostrophe et `--` validés sur le PDF neuf (F10) ; forme du `mariadb-dump` et de la restauration aux
    Dev Notes, rejouées au T7 (F11) ; message du test 11 qui nomme l'umask (F12) ; angle mort SELinux.
  - **Recompte** (depuis la fiche, `grep`) : **16 AC**, **11 tâches** (T0–T10), **19 lignes de test** (18
    → 19) portant **19 fonctions Rust** (15 neuves, 4 existantes complétées) dans un fichier neuf et cinq
    existants, plus 1 étape CI et un module d'aide de test ; **40 mutations** (M1–M40, 37 → 40, aucun trou
    ni doublon). **Modules de code** : 5 (`config`, `main`, `routes/admin`, `errors`, catalogues
    `kesh-i18n`) — inchangé : la remédiation n'ajoute que des fichiers d'exclusion, de documentation, un
    helper de test et un retrait de script ; la convention de décompte est désormais écrite (§ *Règle de
    découpage*).
  - **Signal de découpage** (D5) : sévérité égale (MEDIUM → MEDIUM). Les six MEDIUM sont **distincts** de
    ceux de la P1 ; deux naissent de la remédiation P1 (R2-1 : test 16 ; R2-2 : registres non propagés) et
    un en est aggravé (F2 : `init-demo.sh` rendu opérant) ; trois sont des sites d'origine que l'inventaire
    avait manqués (brochure, `.gitignore`, `DOCKER_START.md:134`). Aucun défaut ne revient sous une autre
    forme : pas de recyclage, seuil de modules non franchi — pas de découpage, signal déclaré au Project
    Lead via l'orchestrateur.
- 2026-10-09 — **Validation P3 et découpage** (agent remédiateur, Opus 5.5, en autonomie). Passe P3 : deux
  lentilles **Opus 5.5** en contexte frais (prompt `15-13-validate-prompt-p3.md`) — R (regression hunter)
  **0 CRITICAL / 0 HIGH / 2 MEDIUM / 8 LOW**, F (adversaire plein périmètre) **0 / 0 / 3 MEDIUM / 8 LOW** ;
  aucun doublon MEDIUM — soit **5 MEDIUM distincts** ; 16 LOW, dont 14 distincts (R3-3 = F-P3-4 : #577 ;
  F-P3-7 contenu dans R3-6). **Trend** : P1 (Opus 5.5 ×2) 8 MEDIUM distincts → P2 (Sonnet ×2) 6 → P3 (Opus 5.5
  ×2) 5 ; 0 CRITICAL/HIGH aux trois passes.
  - **Signal D5 de recyclage, deuxième passe consécutive** : en P2, deux MEDIUM nés de la remédiation P1
    (R2-1, R2-2) ; en P3, trois nés de la remédiation P2 — **R3-1** (correctif de F1 : le test 19 admettait
    le motif non ancré que C-15-13-14 écarte, et `backup/` masque `frontend/src/routes/(app)/admin/backup/`,
    versionné), **R3-2** (correctif de F2 : le grep de vérification du retrait d'`init-demo.sh` contredit
    l'entrée de CHANGELOG exigée), **F-P3-2** (correctif de R2-9 non propagé à `CHANGELOG.md:52`). Selon
    l'amendement D5 (rétrospective de l'Epic 25), c'est la forme qui découpe : défauts nés du correctif
    précédent. **Décision de l'orchestrateur : découpage** (C-15-13-16) selon la couture #551 / #552-#576.
  - **Répartition** : § *Découpage* ci-dessus ; recompte aux deux bornes écrit. Numérotation de la fiche
    unique conservée dans les deux filles (AC, tests, mutations, tâches) pour la traçabilité.
  - **Remédiation** (détail dans le Change Log de chaque fille) : 15-13a — R3-2, F-P3-1 (partie MariaDB),
    F-P3-2, F-P3-3 (C-15-13-17, avertissement du mot de passe applicatif gabarit, M42 ; angle mort root
    gabarit → issue à ouvrir par l'orchestrateur), LOW R3-3/F-P3-4, R3-4, R3-5, R3-6 (partie a), R3-7,
    R3-8, F-P3-6, F-P3-8, F-P3-9 ; 15-13b — R3-1 (C-15-13-19, M41), F-P3-1 (partie sauvegarde), LOW R3-6
    (partie b)/F-P3-7, R3-9, F-P3-5, F-P3-10, F-P3-11. **R3-10** (`sprint-status.yaml`) : en-tête `(27)`
    rétabli dans sa forme d'origine, entrée `(28)` ajoutée. Rectificatifs du registre (C-15-13-13 : `DELETE`
    à `:70-75` ; C-15-13-14 : homonyme réel et ancrage ; C-15-13-15 : #577) : C-15-13-20.
  - **Constats du remédiateur, hors rapports** : M5 et `VALEURS_COMPOSEES` relèvent du test `valeurs`
    (famille (V)), non de `transmission` — ligne 3a de la 15-13a, fonction `valeurs` comptée ; grep
    `MARIADB_` (axe non exercé par la lentille R de la P3) rejoué : quatre homonymes (`retry.rs:57`, `:106`,
    `reconciliation.rs:1129`, `:1136`) entrés à l'inventaire de la 15-13a.
- 2026-10-09 — **Validation P4 des deux filles** (agent remédiateur, Opus 5.5). Deux lentilles **Sonnet** par
  fille : 15-13a **1 MEDIUM** distinct (R4-1 = F1), 15-13b **4 MEDIUM** distincts (R4-2 = F-P4-1) ; 0
  CRITICAL/HIGH. Remédiation dans chaque fille (Change Log) ; choix C-15-13-21 à C-15-13-26. Ici : ligne
  « Inventaire » du recompte réduite aux totaux (R4-7).
- 2026-10-09 — **Validation P5 de la 15-13b** (agent remédiateur, Opus 5.5). Deux lentilles **Opus 5.5** :
  **2 MEDIUM** distincts (R5-1, né de la remédiation P4 ; F-P5-1, d'origine P1), 13 LOW (12 distincts : R5-3 = F-P5-8) ; 0 CRITICAL/HIGH.
  Trend de la 15-13b : P4 4 → P5 2. Remédiation dans la fiche fille (Change Log) ; choix C-15-13-27 à
  C-15-13-29. Ici : union des mutations des deux filles portée à **51** (M1 à M51 : la 15-13b ajoute M47 à
  M51) ; la phrase « deux gestes » de `CHANGELOG.md:46` est **partagée** entre les deux filles (R5-5,
  ventilation écrite dans la 15-13b seule).

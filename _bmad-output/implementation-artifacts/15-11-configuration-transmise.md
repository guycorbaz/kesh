# Story 15.11 : La configuration écrite dans `.env` atteint Kesh — fiche index, découpée en 15-11a et 15-11b

Status: split

<!-- Découpée le 2026-10-08 après la validation P3 (signal D5 levé deux fois, par recyclage), décision de
     l'orchestrateur — choix C77 (`epic-15-choix-autonomes.md`). Corps de la fiche vidé : la **version
     complète avant découpage** est au commit **d69fdcca** (`git show d69fdcca:_bmad-output/implementation-artifacts/15-11-configuration-transmise.md`).
     Le Change Log des validations P1 à P3 est conservé ci-dessous ; la remédiation P3 est appliquée dans
     les deux fiches filles. -->

**Issue** : #550 (fermée par la **15-11a**, qui ferme aussi #557) ; `refs #534`, `refs #551`, `refs #552`, `refs #558`.

## Découpage

| fiche | contenu | issue | dépend de |
|---|---|---|---|
| **15-11a** — `15-11a-compose-transmet-la-configuration.md` | ajouts dans les deux compose de production (`${KESH_X:-}`, `KESH_LOG_FILE_PATH` en `${X-défaut}`, `KESH_ADMIN_PASSWORD` sans défaut), `image:` dans `docker-compose.yml`, montages de `docker-compose.prod.yml` **inchangés** (AC4 abandonnée en P3, C83 — version configurable : #558), garde des placeholders `GENERATE_ME` et `<…>` du secret JWT et du mot de passe admin (AC16, #557), fantôme `KESH_ADMIN_RESET`, `.env.example`, manuel admin, `DOCKER_START.md`, `restart` → `up -d`, « relisez votre `.env` », gestes de mise à jour, CHANGELOG **Corrigé**, `docker compose config -q` en CI ; test `configuration_transmise.rs` **simple**, qui ne lit pas le code Rust : liste fermée `LUES` (41 variables, écrite en dur, reproduite par un `grep` documenté), familles (T), (V), (E), (F) sur les corpus texte, (S) ; comptes à la clôture de sa validation (P4, C84) : 16 AC, 10 tâches, 28 mutations — la fiche fait foi | **closes #550**, **closes #557**, refs #534, #558 | — |
| **15-11b** — `15-11b-lecture-unique-des-variables.md` | fonction `config::env_nonempty` (vide = absent, trim), migration des 36 sites de lecture, test qui lit le code avec `syn` (règle (L), `INDIRECTIONS`, `cfg(test)`, macros ; (F) étendu aux littéraux du code et des macros) et qui **remplace** `LUES` ; `proc-macro2` en dev-dépendance ; CHANGELOG **Modifié** ; 6 AC, 7 tâches, 11 mutations | refs #550 | 15-11a **et** 15-5e1 (`syn`) |

**Ordre écrit** : 15-11a → 15-11b (après la 15-5e1). **La 15-7b2 dépend de la 15-11a** (et d'elle seule) :
ordre de merge 15-11a avant 15-7b2 (C-15-7-51). La 15-11a se merge seule sans danger (aucune variable
ajoutée ne fait refuser le démarrage quand elle arrive vide — vérifié au code, § *Le vide avant la 15-11b*
de sa fiche) ; la 15-11b est attendue avant le tag v0.13.0.

## Change Log

- 2026-10-08 — Création (agent de spécification, autonomie). Inventaire refait depuis le code : 41
  variables lues, 16 à transmettre (28 lignes), 2 fixées par l'image, 1 interdite, 3 d'hôte (ignorées
  par `docker-compose.prod.yml`), 4 MariaDB, 1 fantôme (`KESH_ADMIN_RESET`). Défaut connexe établi :
  `docker-compose.yml` sans `image:` (installation du manuel en échec, vérifié). Choix C71, C72, C73.
- 2026-10-08 — **Validation P1** (lentilles R et F, Sonnet ×2 ; rapports `target/gate-logs/15-11-p1-{R,F}.md`) :
  **0 CRITICAL, 0 HIGH, 13 MEDIUM, 13 LOW** bruts (R : 6 MEDIUM, 7 LOW ; F : 7 MEDIUM, 6 LOW), dont 5 paires
  convergentes (R2 = F6, R3 = F2, R4 = F4, R6 = F5, R1 ≈ F8) — **8 MEDIUM distincts**. Tous traités :
  - **Forme de transmission (R2/F6, R3/F2, F1)** : abandon de la clé sans valeur (ne protège pas du vide,
    mesurée sur une seule version de Compose) ; les ajouts en `${KESH_X:-}` ; **fonction de lecture unique**
    `config::env_nonempty` (vide = absent, trim), imposée à 36 sites par (L) — **AC15 neuf** ; AC2, AC8 (V),
    AC10 (comptes revus : 38 clés présentes, 23/15 et 22/16 non vides/vides, aucune `null`) ; T0 mesure la
    version de Compose. **C75 révise C71.**
  - **`KESH_LOG_FILE_PATH` (F3)** : l'opt-out documenté était sans effet ; forme `${…-défaut}` sans
    deux-points, liste `VIDE_SIGNIFIANT` contrôlée par (V) ; recensement des autres « vide » : aucun autre.
  - **(L) (R1, F8)** : imports de `std::env::var*` et renommages rougissent ; chemin non appelé compté ;
    règle `cfg(test)` écrite (items, instructions, `all`/`any`/`not`/`cfg_attr`, modules hors ligne) — R11.
  - **Propagation `restart` → `up -d` (R4/F4)** : **AC16 neuf** (`admin-manual.tex:1239`, `:1289`,
    `DOCKER_START.md:62`, `:106`, grep élargi) ; recette de la 15-7b2 laissée à la 15-7b2 rebasée.
  - **Garde-fous structurels (R5)** : `image:` de Y et montages de P dans (T) ; M18, M19.
  - **Mise à jour (R6/F5)** : bloc exact au manuel, avertissement « relisez votre `.env` » avec la liste
    nommée ; CHANGELOG idem, plus une entrée **Modifié** (vide = absent, trim).
  - **Coordination 15-7b2 (F7)** : ordre de merge (15-11 d'abord), hunks voisins de `sec:env-vars`, règle
    de non-recouvrement.
  - **LOW** : précondition, repli et effet développeur de l'AC5 (R7, F11) ; `PREFIXES_HORS_VARIABLES`
    retirée (R8) ; `KESH_HOST` n'est pas une exception, `KESH_TEST_MODE` contrôlée dans les deux sens (R9,
    F9) ; AC10 et `DATABASE_URL`/`KESH_HOST` (R10, F10) ; #551/#552 citées, écart avec #550 déclaré (R12) ;
    « 16 variables / 28 lignes » et deux formes par variable (R13) ; limites de `yaml-rust2`, `hashlink`,
    absence de `deny.toml`, résolution du conflit `Cargo.lock` avec la 15-5e1 (F12) ; « transmise ≠
    effective » (F13).
  - Mutations **16 → 21** (M12 redéfinie, M17-M21 neuves ; 20 rouges, 1 verte) ; AC **14 → 16** ; tâches
    inchangées en nombre (T0-T9), T0-T3, T5, T6, T8 amendées.
  - **Règle de découpage** : un crate, cinq modules de code — seuil non franchi ; coupe 15-11a/15-11b
    signalée, non appliquée.
  - **Signal D5** : sans objet (première passe).
  - Symptômes grepés dans la fiche (`clé sans valeur`, `M1-M16`, `KESH_PAT_`, `PREFIXES_HORS`,
    `n'apparaissent pas`, `aucune chaîne vide`, `supprime la question`, `ajouter à la main`, `pas de
    chevauchement`) : occurrences restantes toutes historiques ou voulues.
- 2026-10-08 — **Validation P2** (lentilles R et F, Opus ×2 ; rapports `target/gate-logs/15-11-p2-{R,F}.md`,
  prompt `15-11-validate-prompt-p2.md`) : **0 CRITICAL, 0 HIGH, 9 MEDIUM, 17 LOW** bruts (R : 5 MEDIUM, 7 LOW ;
  F : 4 MEDIUM, 10 LOW), dont 3 paires convergentes (R2-2 = F2, R2-3 = F3, R2-5 ≈ F5, ce dernier LOW côté F)
  et 2 LOW convergents (R2-6 = F7) — **7 MEDIUM distincts**. Plus **1 MEDIUM trouvé à la remédiation**
  (O-1, ci-dessous). Tous traités (décisions de l'orchestrateur, C76) :
  - **F1** — `KESH_ADMIN_PASSWORD: ${…:-changeme}` dans Y rendait l'onboarding `/setup` impossible et
    faisait refuser le démarrage après un break-glass ; la fiche l'écartait par un argument faux
    (« un mot de passe vide suffit ») : corrigé (§ *Inventaire*, angles morts). Y passe à `${…:-}` (AC2 i),
    vérifié au code (`config.rs:619-637`, `auth/bootstrap.rs:62-69` : absent → `None` → setup-required,
    sans refus) ; P (`${…}`) inchangé ; `KESH_ADMIN_USERNAME` reste `:-admin` (couple exigé). Liste fermée
    `SANS_DEFAUT` en (V), mutation **M22**. Risque `/setup` écrit comme comportement existant et documenté.
    AC10 (ii) Y : 23/15 → **22/16**.
  - **R2-1** — (L) compte aussi comme noms lus les arguments littéraux des **sites directs** (qui restent
    rouges) ; entrée transitoire `opt_trimmed_env` d'`INDIRECTIONS` du T1 au T3 ; le T1 écrit **exactement**
    le rouge attendu par assertion, et l'ensemble lu attendu (41 = 32 + 2 + 1 + 5 + 1).
  - **R2-2 = F2** — `admin-manual.tex:1306` (AC12 i) et `.env.example:110-111` (AC6 g) entrent dans la
    propagation du trim ; `espaces` au grep du T8 et à l'aplati de l'AC12 (g).
  - **R2-3 = F3** — la liste « relisez votre `.env` » (AC12 f, AC13) est **fermée** depuis les variantes de
    `ConfigError` atteignables par une variable nouvellement transmise : `InvalidBoolValue`
    (`KESH_SMTP_TLS`, `KESH_FEATURE_FORGOT_PASSWORD`), `InvalidCookieSecureValue` (P), `IncompleteSmtpConfig`
    (absente, `KESH_SMTP_FROM` mal formé, `KESH_SMTP_HOST` avec port) ; effets sans refus (envoi de factures
    activé par les quatre `KESH_SMTP_*`, F9 `MIN_PASSWORD`) ; constat rassurant (gabarit inchangé).
  - **R2-4** — le « bloc exact » devient **trois gestes** (ajouter, remplacer `KESH_LOG_FILE_PATH` et
    `KESH_ADMIN_PASSWORD`, remplacer les montages de P) et `docker compose config -q` avant `up -d` ; la
    voie recommandée reste le re-téléchargement.
  - **R2-5 (≈ F5)** — le commentaire `.env.example:206-207` est **réécrit** (AC6 d : absent → activé sous
    Docker) ; (V) exige le marqueur « contrairement aux autres variables » et refuse « ou absent » ; (S)
    l'exerce. Numéros `:205-206` → `:206-207`.
  - **F4** — (L) traite `dotenvy::var*` comme site, interdit son import (glob, renommage), (S) et mutation
    **M23** ; `TMPDIR` (`temp_dir`) écrit comme angle mort.
  - **O-1 (MEDIUM, trouvé à la remédiation en écrivant le rouge exact du T1)** — (F) ne lisait que les
    `syn::LitStr` ; or `config.rs:158` est dans un `write!` et `main.rs:337` dans un `tracing::info!` :
    `syn` ne voit pas l'intérieur des macros, (F) ne les aurait pas vus et **M11 serait restée verte**. (F)
    parcourt désormais les littéraux de chaîne du flux de jetons des macros ; (S) l'exerce.
  - **LOW** : non-UTF-8 perdu avant l'abonné, limite écrite (R2-6 = F7) ; `env -i` à l'AC10 (R2-7) ;
    AC12 (c) vers `:783-784`, pas le `\paragraph` (R2-8) ; les 16 ajouts tous en `${KESH_X:-}` (R2-9) ;
    aucun commentaire des compose ne nomme `KESH_PRODUCTION_RESET` (R2-10, contrôle du T8 de la 15-7b2) ;
    `dotenvy::dotenv()` à deux endroits (R2-11) ; le test n'écrit pas le fantôme en un jeton, grep de
    l'AC7 sans exclusion (R2-12) ; texte de l'AC12 (b) en colonne Description (F6) ; changements de sens
    du vide nommés, dont `RUST_LOG=""` au CHANGELOG (F8) ; `reset_env()` complété (F10) ; effet inverse
    `--build` / `docker compose pull` (F11) ; Container Manager non mesuré (F12, AC12 j) ; `- KEY` sans `=`
    rouge (F13) ; `docker-compose.dev.yml` sans `.env` — angle mort, **issue à ouvrir** (F14).
  - Comptes **recomptés depuis la fiche** : AC **16** (inchangé ; sous-points AC2 i/ii, AC6 g-h, AC12 i-j
    neufs), tâches **10** (T0-T9 ; T1, T2, T3, T5, T8 amendées), mutations **21 → 23** (M22, M23 ; 22
    rouges, 1 verte), modules de code **5** (inchangé — seuil de découpage non franchi).
  - **Signal D5, déclaré au Project Lead** : sévérité **MEDIUM → MEDIUM** (P1 : 8 MEDIUM distincts ; P2 :
    7 + 1). Plusieurs défauts **naissent de la remédiation P1** — R2-1 (déplacement de l'extraction des
    noms vers la fonction unique), R2-2 (trim), R2-4 (bloc à coller), R2-5 (contrôle (V) satisfait par le
    commentaire faux) ; les autres (F1, F3, F4, O-1) sont des défauts **d'origine** distincts. Recyclage
    localisé, traité sur place ; dispersion inchangée (un crate, cinq modules) : **pas de découpage**
    (amendement D5).
  - Symptômes grepés dans la fiche (`sans conséquence`, `le dit déjà`, `mot de passe vide suffit`, `bloc
    exact`, `trois lignes qui précèdent`, `205-206`, `:782`, `M1-M21`, `20 rouges`, `sauf une`) et sur le
    dépôt (`espace` + refus, `VIDE ou absent`, `changeme` dans les compose et `.env.example`) : restes
    historiques (Change Log P1) ou sites pris en charge par un AC.
- 2026-10-08 — **Validation P3** (lentilles R et F, Sonnet ×2 ; rapports `target/gate-logs/15-11-p3-{R,F}.md`,
  prompt `15-11-validate-prompt-p3.md`) : **0 CRITICAL, 0 HIGH, 6 MEDIUM, 13 LOW** bruts (R : 2 MEDIUM,
  7 LOW ; F : 4 MEDIUM, 6 LOW), dont 2 paires convergentes (R3-1 = F-1, MEDIUM des deux côtés ; R3-3 = F-3,
  LOW côté R, MEDIUM côté F) et 1 LOW convergent (R3-8 = F-7) — **4 MEDIUM distincts** (R3-1 = F-1, R3-2,
  F-2, F-3 = R3-3) **plus F-4**, MEDIUM de processus (signal D5). Défauts **nés d'une remédiation** : R3-1 =
  F-1 (le « rouge exact » du T1 écrit en P2), F-2 et F-3 (le parcours des macros de (F) ajouté en P2, O-1) ;
  R3-2 est d'origine.
  - **Signal D5, levé pour la deuxième fois, par recyclage** (F-4) : P1 → P2, 4 des 8 MEDIUM distincts de
    P2 naissaient de la remédiation P1 (R2-1, R2-2, R2-4, R2-5) ; P2 → P3, 3 des 4 MEDIUM distincts de P3
    naissent de la remédiation P2 (R3-1 = F-1, F-2, F-3), et ils se concentrent dans la **machinerie du test
    qui lit le code** (règles L et F, `syn`/`proc_macro2`, définition du rouge exact). L'exception de
    l'amendement D5 (« défauts distincts **et** qui ne viennent pas d'une remédiation ») ne s'applique pas.
    **Découpage décidé par l'orchestrateur** (C77) : 15-11a (compose, documentation, test simple à liste
    fermée — ferme #550, débloque la 15-7b2) et 15-11b (fonction de lecture unique et test qui lit le code).
  - **Remédiation P3, appliquée dans les fiches filles** :
    - 15-11a : **R3-1 = F-1** (rouge exact du T1 de (E) : les trois `KESH_*_HOST_DIR`, pas la prose
      `.env.example:235` ; NOM = `[A-Z][A-Z0-9_]*`) ; **R3-2** (refus de `main.rs:316-323` quand le mailer
      ne se construit pas avec le mot de passe oublié ; activation de la réinitialisation par e-mail dans les
      effets ; `process::exit` relevés au T0) ; **F-10** (le défaut `change-me…` du secret JWT, refusé par
      `InsecureJwtSecret` — `config.rs:35-39`, vérifié par l'orchestrateur — est une garde, conservée ; une
      installation sans `.env` ne démarre pas, `KESH_JWT_SECRET` obligatoire écrit) ; LOW R3-4, R3-6 (M3,
      M7, M14), R3-8 = F-7, R3-9, F-5, F-6, F-8.
    - 15-11b : **F-2** (exclusion `cfg(test)` de (F)) ; **F-3 = R3-3** (`proc-macro2` en dev-dépendance) ;
      LOW R3-6 (M1, M3), R3-7 (site d'une indirection après bascule), F-9 (« hors Docker » pour
      `RUST_LOG=""`).
    - Registre : **R3-5** (C75 admettait `${KESH_X:-défaut}` pour un ajout ; précisé dans C77 : tous les
      ajouts en `${KESH_X:-}`, R2-9 ; titres de C71 et « seize mutations » de C72 historiques).
  - Comptes du découpage, **recomptés depuis les fiches filles** : 15-11a — 15 AC, 10 tâches, 19 mutations
    (toutes rouges), 3 modules de code (textes seulement) ; 15-11b — 6 AC, 7 tâches, 11 mutations (10
    rouges, 1 verte), 4 modules de code. Seuil de découpage (> 5 modules) non franchi par l'une ni l'autre.
    *(15-11 avant découpage : 16 AC, 10 tâches, 23 mutations, 5 modules.)*
- 2026-10-08 — **Ligne 15-11a du découpage mise à jour** (R4-2 de la validation P4 de la 15-11a, choix
  C84) : montages de `docker-compose.prod.yml` inchangés (AC4 abandonnée en P3, C83, #558), AC16
  (#557), `closes #550` **et** `closes #557`, comptes à la clôture — 16 AC, 10 tâches, 28 mutations (la
  fiche 15-11a fait foi). Validation de la 15-11a close en P4 (0 au-dessus de LOW).

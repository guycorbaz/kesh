# Story v017-6 : deux petites corrections — le refus qui se répète (#832) et la langue de l'interface au manuel (#824)

Status: ready-for-dev

*Fiche rédigée le 2026-10-31, en autonomie (Sonnet 5.5), worktree `/home/gcorbaz/devel/kesh-0177`, branche
`story/v017-6-petites-corrections` partie de `8e126cfd` (tête de `release/0.17.0`). Release **0.17.0**. Registre :
`v017-6-choix-autonomes.md` (C-v017-6-1 à 8). Clé courte : `v0176`. Issues : **#832** (bug, P4) et **#824** (doc, P3),
en `refs` seulement dans les commits ; la fermeture se porte sur la PR. Les numéros de ligne sont ceux de `8e126cfd`.*

## Story

En tant qu'Administrateur qui saisit un numéro IDE ou un IBAN faux, je veux un refus qui dise la chose une fois
(« Numéro IDE invalide : 9 chiffres attendus après CHE, trouvé 3 »), et, en tant que lecteur du manuel utilisateur, je
veux savoir d'où vient la langue de l'interface (l'installation) et ce que règle l'assistant (la langue des documents),
afin de ne pas chercher à l'assistant un réglage qu'il n'a pas.

## Origine

- **#832 (P4)**, constaté au développement de la v017-2 (#809, 2026-10-30) : depuis que l'assistant d'installation
  affiche le motif du serveur, le refus d'un IDE invalide se lit « IDE invalide : Numéro IDE invalide : 9 chiffres
  attendus après CHE, trouvé 3 ». Remède de l'issue : ne pas préfixer, ou préfixer par le nom du champ seulement.
- **#824 (P3, doc)**, finding F2 de la validation P2 de la v017-2 : le § *Choix de la langue* du manuel utilisateur dit
  la langue « choisie lors de l'onboarding » ; elle est celle de l'installation.
- **Données à garder, migration : aucune.** Deux messages de refus et une page de manuel. Aucune table, aucune route,
  aucune migration, aucun bump ; l'inventaire des tables est inchangé (contrainte du Project Lead du 2026-10-30,
  `kesh-gate-logs/prompts/v017-donnees-a-garder.md`). Tests sur bases `#[sqlx::test]` jetables ; rien ne se joue sur le
  NAS.

## Ce que le code dit (relevé le 2026-10-31, `8e126cfd`)

### #832

1. **Confirmée** — `crates/kesh-api/src/routes/onboarding.rs:538` :
   `.map_err(|e| AppError::Validation(format!("IDE invalide : {e}")))?;`, et `crates/kesh-core/src/errors.rs:25` :
   `#[error("Numéro IDE invalide : {0}")] InvalidCheNumber(String)`. `CheNumber::new` (`che_number.rs:83-127`) ne rend
   que des `CoreError::InvalidCheNumber`. Le message servi est donc « IDE invalide : Numéro IDE invalide : … ».
2. **Précisée : la classe est plus large que l'IDE.** `errors.rs:17-23` porte la même forme pour l'IBAN
   (`#[error("IBAN invalide : {0}")] InvalidIban`) et le QR-IBAN (`#[error("QR-IBAN invalide : {0}")] InvalidQrIban`),
   et **quatre** sites les préfixent à leur tour du même mot (relevé par
   `grep -rnE 'format!\("[^"]*invalide :? ?\{' crates --include=*.rs`) : l'IBAN et le QR-IBAN de l'assistant
   (`routes/onboarding.rs:601, 607`) et ceux de la création d'un compte bancaire (`routes/bank_accounts.rs:320, 327`).
   Ils rendent « IBAN invalide : IBAN invalide : trop court ». Défaut identique, remède identique, un fichier de plus :
   **intégré** (C-v017-6-1), non ouvert en issue distincte.
3. **Les sites voisins qui ne sont pas du même défaut** — inventaire clos des sites qui préfixent le `Display` d'un
   `CoreError` ou d'un type voisin, du `grep` ci-dessus et de `grep -rn 'CheNumber::new\|Iban::new\|QrIban::new' crates/kesh-api/src` :

   | Site | Verdict |
   |---|---|
   | `routes/onboarding.rs:538` (IDE) | **à corriger** |
   | `routes/onboarding.rs:601, 607` (IBAN, QR-IBAN) | **à corriger** |
   | `routes/bank_accounts.rs:320, 327` (IBAN, QR-IBAN) | **à corriger** |
   | `routes/contacts.rs:322` (IDE) | conforme : jette l'erreur (`\|_\|`), message propre « Numéro IDE suisse invalide (format ou checksum) » |
   | `routes/supplier_invoices.rs:404` (IBAN) | conforme : branche sur la variante, n'interpole pas le message |
   | `routes/journal_entries.rs:539, 542` ; `routes/opening_balances.rs:364, 367` ; `services/ecritures.rs:87, 99` ; `services/postes_ouverts.rs:67` | hors classe : l'erreur interpolée est celle de `Decimal`/`NaiveDate` (bibliothèque), non un `CoreError` qui porte déjà « invalide » |
   | `routes/onboarding.rs:163, 202, 378, 423` ; `routes/profile.rs:35` | hors classe : interpolent la **valeur saisie**, pas un message d'erreur |
   | `routes/supplier_invoices.rs:833`, `routes/bank_imports.rs:875`, `routes/admin.rs:296, 315`, `errors.rs:2689` | hors classe : le mot préfixé n'est pas celui du message interpolé |

   Les autres variantes de `CoreError` (`errors.rs:29-70`) ne sont préfixées par aucune route (le `grep` ne rend rien).
   Aucun site non résolu : chaque ligne du `grep` est classée.
4. **Aucun test n'épingle la répétition.** `crates/kesh-api/tests/bank_accounts_e2e.rs:2131, 2135` n'exigent que
   `message.starts_with("IBAN invalide")` / `"QR-IBAN invalide"` : **vrais avant et après** le correctif, donc muets
   contre la régression. `onboarding_path_b_e2e.rs:200-211` (IDE `"INVALID"`) ne lit que le code `VALIDATION_ERROR`.
   Les trois restent verts, inchangés, et ne suffisent pas : la story ajoute des assertions d'**égalité exacte**
   (AC-2).
5. **Le frontend ne figure pas le texte** : `grep -rn "IDE invalide" frontend/src frontend/tests` ne rend que
   `frontend/tests/e2e/contacts.spec.ts:69` (titre d'un test des contacts, route distincte, regex `/IDE|invalid/i` qui
   reste vraie). Aucune clé i18n, aucune des quatre locales n'est touchée : le message est écrit en dur côté serveur
   (#710, hors périmètre).

### #824

6. **Confirmée** — `docs/manual/fr/user-manual.tex:143-153`, § *Choix de la langue* : « Kesh gère le français,
   l'allemand, l'italien et l'anglais pour les messages du système. La langue est choisie lors de l'\textbf{onboarding}. »
   puis, dans un `keshnote` : « … dans la version actuelle, la langue est fixée à l'onboarding et n'est pas modifiable
   depuis un menu. »
7. **Ce que le code fait** : l'interface est servie dans la locale de l'installation —
   `routes/i18n.rs:20-22` (`let locale = &state.config.locale;`), remplie depuis `KESH_LANG` (`config.rs:845-848`,
   `env_nonempty("KESH_LANG").unwrap_or_else(|| "fr".into())`). L'étape 0 de l'assistant (`set_language`,
   `routes/onboarding.rs:155-175`) ne règle que `instance_language` de la société : la **langue des documents** — celle
   de la facture, du PDF figé et de l'e-mail quand le contact n'a pas sa propre langue
   (`routes/invoice_email.rs:71`, `contact.language.unwrap_or(company.instance_language)` ; `routes/issued_invoice_pdf.rs:342`).
   Le frontend recharge les messages après le choix (`+page.svelte:44-47`, `loadI18nMessages`) mais le serveur les sert
   dans la locale de l'installation : le choix ne change pas l'interface.
8. **Le manuel d'administration le dit correctement** : `admin-manual.tex:757`, tableau *Serveur \& réseau* —
   « KESH\_LANG & Locale de l'instance (fr/de/it/en). & fr ». Le registre est exact mais **laconique** : il ne dit ni
   que c'est la langue de l'interface, ni que l'assistant n'y touche pas (C-v017-6-5).
9. **Inventaire clos des phrases fausses** (`grep -n -i "langue.*onboarding\|onboarding.*langue\|choisie.*langue\|langue.*choisie\|fixée"` sur
   le manuel, le README et le site) — **quatre** sites, non trois :

   | Site | Texte | Verdict |
   |---|---|---|
   | `user-manual.tex:146` | « La langue est choisie lors de l'onboarding. » | **faux** — à récrire |
   | `user-manual.tex:149-152` | note « fixée à l'onboarding et n'est pas modifiable depuis un menu » | **faux** — à récrire (le sélecteur reste à venir ; ce qui est faux, c'est l'origine) |
   | `user-manual.tex:3732` | limites : « Sélecteur de langue dans l'interface (la langue est fixée à l'onboarding … ) » | **faux** — à récrire, **non nommé par l'issue** |
   | `README.md:53` | « messages d'erreur API en FR/DE/IT/EN (langue choisie à l'onboarding ; sélecteur … à venir) » | **faux** — même défaut, **non nommé par l'issue** ; intégré (C-v017-6-3) |
   | `user-manual.tex:183-184, 201, 1140, 3392` ; `admin-manual.tex:1069, 1418` | « langue de l'installation », « langue comptable », « langue de l'instance », liste des étapes | conformes |
   | `website/*.html`, `marketing-brochure.tex`, `docs/manual/{de,en,it}/README.md` | aucune occurrence | sans objet |

   Les manuels DE/IT/EN ne sont que des `README.md` (« à traduire ») : aucune traduction du § à tenir.
10. **L'incise de la v017-2** : la même sous-section porte, dans le `keshwarning` (l. 156-163), « certains refus du
    serveur restent toutefois rédigés en français (\#710) ». **Elle est juste et ne bouge pas** : elle parle des refus de
    validation écrits en dur, indépendants de la locale. Le correctif de #832 ne la touche pas. (La phrase de l'étape 4
    de l'assistant, l. 202, qui cite le motif « qui peut être rédigé en français quelle que soit la langue de
    l'interface », reste également vraie.) Elle ne doit pas être **contredite** par la récriture : la phrase neuve dit
    que l'interface est dans la langue de l'installation, et la mise en garde dit que certains refus échappent à cette
    langue — les deux tiennent ensemble.

## Critères d'acceptation

### #832 — le refus qui se répète

- **AC-1 — Un seul préfixe.** Les cinq sites du tableau (§ 3) rendent `AppError::Validation(e.to_string())` : le message
  du `CoreError`, tel que `thiserror` le forme, sans second préfixe. Le doc-comment du site de l'assistant
  (`onboarding.rs:535`, « Validate IDE via kesh-core if provided ») dit que le message de `kesh-core` porte déjà le
  nom du champ. Le motif lu par l'utilisateur devient « Numéro IDE invalide : 9 chiffres attendus après CHE, trouvé 3 »,
  « IBAN invalide : trop court », « QR-IBAN invalide : QR-IID 12345 hors plage 30000-31999 ».
- **AC-2 — Le message est figé, à l'égalité exacte.** Quatre assertions, **tests rouges d'abord** :
  (a) `onboarding_path_b_e2e.rs`, le test de l'IDE invalide (`:200-211`) : `body["error"]["message"]` **égal** à
  « Numéro IDE invalide : doit commencer par CHE » (entrée `"INVALID"` : `che_number.rs:83`) ;
  (b) un test de l'IBAN invalide de l'étape banque de l'assistant (`POST /api/v1/onboarding/bank-account`, IBAN `"BAD"`) :
  message **égal** à « IBAN invalide : trop court » (`iban.rs:148`, longueur < 5) — ou la valeur que le développeur relève
  au rouge, **recopiée du serveur d'avant correction moins le préfixe**, jamais devinée ;
  (c) le même pour le QR-IBAN de l'assistant (IBAN valide, QR-IBAN `"BAD"`) ;
  (d) `bank_accounts_e2e.rs`, `ordre_des_refus_de_forme` (`:2111-2147`) : en plus du `starts_with`, une assertion
  que `message.matches("IBAN invalide").count() == 1` pour les deux cas IBAN/QR-IBAN (« QR-IBAN invalide » ne contient
  « IBAN invalide » qu'**une** fois ; le message neuf n'en a qu'un, l'ancien en avait deux). Le `starts_with` existant
  reste, inchangé.
  Les entrées sont des fixtures synthétiques ; aucune donnée réelle.
- **AC-3 — Mutations (sous le verrou de gate, un site à la fois, tuées une à une).** M1 : remettre
  `format!("IDE invalide : {e}")` à `onboarding.rs:538` → (a) rougit. M2 : remettre `format!("IBAN invalide : {e}")` à
  `onboarding.rs:601` → (b) rougit. M3 : idem `:607` → (c). M4 : idem `bank_accounts.rs:320` → (d) rougit. M5 : idem
  `bank_accounts.rs:327` → (d) rougit. Le développeur consigne pour chacune le test qui rougit ; après chaque
  restauration, **toucher le fichier** (binaire périmé, mémoire `mutation-restauree-binaire-perime`).
- **AC-4 — Aucun autre site ne change.** `git diff --stat` : `routes/onboarding.rs`, `routes/bank_accounts.rs` et les
  deux fichiers de test seulement. Les messages de `contacts.rs:322` et des sites « hors classe » (§ 3) sont
  intouchés. Aucune clé i18n, aucune locale.

### #824 — le manuel

- **AC-5 — Le § *Choix de la langue* est récrit.** Il dit, sans rien promettre de plus : (i) l'interface est servie dans
  **la langue de l'installation**, réglée par l'administrateur (variable `KESH_LANG`, `fr` par défaut ; manuel
  d'administration, tableau des variables d'environnement) ; (ii) l'assistant d'installation, à son premier écran, ne
  règle **que la langue des documents** — celle des factures, des PDF et des courriers, quand le contact n'a pas sa
  propre langue —, non celle de l'interface ; (iii) il n'existe pas de sélecteur : la langue de l'interface ne se change
  pas depuis un menu, et un sélecteur reste prévu pour une version ultérieure. Texte prescrit (le développeur peut
  l'ajuster sans en changer le sens) :
  > Kesh gère le français, l'allemand, l'italien et l'anglais pour les messages du système. L'interface s'affiche dans
  > la \textbf{langue de l'installation}, que fixe l'administrateur (variable \texttt{KESH\_LANG}, français par défaut ;
  > voir le manuel d'administration). Le premier écran de l'assistant d'installation ne règle pas cette langue : il
  > règle celle des \textbf{documents} (factures, PDF, courriers), employée quand un contact n'a pas sa propre langue.
  >
  > `keshnote` : Un sélecteur permettant de changer la langue de l'interface depuis un menu est prévu pour une version
  > ultérieure : dans la version actuelle, elle ne se change pas depuis l'application.

  La `keshwarning` qui suit (traductions non relues, #710, « avant la 0.11.0 ») est **inchangée**.
- **AC-6 — Les trois autres sites suivent** (inventaire § 9) : `user-manual.tex:3732` devient « Sélecteur de langue dans
  l'interface (la langue de l'interface est celle de l'installation, \texttt{KESH\_LANG} ; le sélecteur est prévu pour
  une version ultérieure). » ; `README.md:53` devient « messages d'erreur API en FR/DE/IT/EN (langue de l'installation,
  `KESH_LANG` ; sélecteur de langue dans l'interface à venir) ». Le manuel d'administration n'est **pas** modifié
  (C-v017-6-5).
- **AC-7 — Le contrôle du manuel se fait sur le PDF.** Le développeur régénère le PDF (`make user` dans `docs/manual/`),
  le commit, et contrôle **aplati** :
  `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` contient la phrase neuve sur la langue de
  l'installation, et ne contient plus « La langue est choisie lors de l'onboarding » ni « fixée à l'onboarding »
  (`grep -c` à zéro, recopié au Dev Agent Record). La macro `\keshVersion` n'est **pas** bumpée (gate de release, hors
  story). Le `.tex`, le PDF et le `README.md` voyagent dans le même commit que leur motif.
- **AC-8 — Grep de la valeur, pas de la formulation** (*Propagation post-patch*) : après la récriture,
  `grep -rn -i "choisie lors\|fixée à l'onboarding\|choisie à l'onboarding" docs/manual README.md website` ne rend plus
  aucune ligne ; toute occurrence restante est triée à la main au Dev Agent Record.

### Les deux

- **AC-9 — CHANGELOG.** `CHANGELOG.md`, `## [0.17.0] — Non publié`, section *Corrigé* : deux puces. La première :
  « **Un refus d'IDE, d'IBAN ou de QR-IBAN ne répète plus son mot** ([#832](…)) : « Numéro IDE invalide : … » et non
  « IDE invalide : Numéro IDE invalide : … », à l'assistant d'installation comme à la création d'un compte bancaire. »
  La seconde : « **Le manuel utilisateur dit d'où vient la langue de l'interface** ([#824](…)) : celle de l'installation
  (`KESH_LANG`) ; l'assistant ne règle que la langue des documents. » Les liens sont ceux des puces voisines.
  L'intro « La 0.17.0 réunit six stories » (l. 13) est **laissée** : son décompte appartient à la PR de release.
- **AC-10 — Aucun mot-clé de fermeture** dans les messages de commit (`refs #832`, `refs #824` seulement ; contrôle par
  `git log origin/main..HEAD --format=%B | grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'` vide).
- **AC-11 — Le gate.** `fmt` + `clippy` workspace, puis **gate complet** au dernier commit de code (`kesh-api` touché :
  `scripts/test-fast.sh` sous le verrou, base remise à zéro d'abord). Le Dev Agent Record n'affirme que ce qui a tourné.
  Pas de Vitest ni d'E2E exigés : aucun fichier de `frontend/` ne change (`git diff --stat` le montre). Pas de capture :
  aucun contrôle visible n'est ajouté ni modifié (le toast de l'assistant affiche un texte plus court).

## Tâches

1. Tests rouges d'abord (AC-2), commit « tests rouges ». 2. Correctif des cinq sites (AC-1), mutations M1 à M5 (AC-3).
3. Manuel `.tex`, PDF, `README.md` (AC-5 à AC-8). 4. CHANGELOG (AC-9). 5. Gate complet (AC-11), Dev Agent Record,
`sprint-status.yaml`.

## Dérogation règle de splitting

Aucune : deux modules (`kesh-api`, documentation), deux défauts mécaniques indépendants dans une même release. Le
critère « plus de cinq modules » n'est pas atteint.

## Issues à ouvrir ou à commenter

- **#832** : commenter que la classe couvre aussi l'IBAN et le QR-IBAN (quatre sites de plus, § 3) et que la story les
  intègre.
- **#824** : commenter que deux sites s'ajoutent à celui que l'issue cite (`user-manual.tex:3732`, `README.md:53`).
- Aucune issue neuve : le message reste en français en dur (#710, déjà ouverte) ; rien d'autre n'est écarté.

## Change Log

- 2026-10-31 — fiche créée (Sonnet 5.5, autonomie). Prémisses de #832 et #824 établies au code ; deux extensions
  mécaniques (IBAN/QR-IBAN ; `user-manual.tex:3732` et `README.md:53`). Prochaine : validation P1 (Opus, une lentille
  complète R+F, `v017-6-validate-prompt-p1.md`).

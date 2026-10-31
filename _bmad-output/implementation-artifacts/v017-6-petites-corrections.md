# Story v017-6 : deux petites corrections — le refus qui se répète (#832) et la langue de l'interface au manuel (#824)

Status: ready-for-dev

*Fiche rédigée le 2026-10-31 (remédiée le même jour à la validation P1 : voir le Change Log), en autonomie (Sonnet 5.5), worktree `/home/gcorbaz/devel/kesh-0177`, branche
`story/v017-6-petites-corrections` partie de `8e126cfd` (tête de `release/0.17.0`). Release **0.17.0**. Registre :
`v017-6-choix-autonomes.md` (C-v017-6-1 à 10). Clé courte : `v0176`. Issues : **#832** (bug, P4) et **#824** (doc, P3),
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
- **Données à garder, migration : aucune.** Des messages de refus, des libellés de textes (manuels, guide, un libellé de l'écran *Paramètres*). Aucune table, aucune route,
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
   `grep -rnE 'format!\("[^"]*(IDE|IBAN) invalide :? ?\{' crates --include=*.rs`, qui rend exactement cinq lignes, l'IDE compris) : l'IBAN et le QR-IBAN de l'assistant
   (`routes/onboarding.rs:601, 607`) et ceux de la création d'un compte bancaire (`routes/bank_accounts.rs:320, 327`).
   Ils rendent « IBAN invalide : IBAN invalide : trop court ». ⚠️ **Le QR-IBAN n'est pas symétrique** (validation P1, P1) :
   `QrIban::new` appelle `Iban::new(input)?` (`qr_iban.rs:39`) et propage telle quelle l'erreur `InvalidIban` d'un IBAN mal
   formé (`qr_iban.rs:36-37`) : un QR-IBAN `"BAD"` rend « IBAN invalide : trop court », qui désigne le mauvais champ.
   Aux deux sites QR-IBAN, le remède n'est donc pas `e.to_string()` : voir AC-1. Défaut identique, remède identique, un fichier de plus :
   **intégré** (C-v017-6-1), non ouvert en issue distincte.
3. **Les sites voisins qui ne sont pas du même défaut** — inventaire clos des sites qui préfixent le `Display` d'un
   `CoreError` ou d'un type voisin, du `grep` ci-dessus et de `grep -rn 'CheNumber::new\|Iban::new\|QrIban::new' crates/kesh-api/src` :

   | Site | Verdict |
   |---|---|
   | `routes/onboarding.rs:538` (IDE) | **à corriger** |
   | `routes/onboarding.rs:601` (IBAN), `routes/bank_accounts.rs:320` (IBAN) | **à corriger** : `e.to_string()` |
   | `routes/onboarding.rs:607` (QR-IBAN), `routes/bank_accounts.rs:327` (QR-IBAN) | **à corriger** : helper commun (AC-1) |
   | `routes/contacts.rs:322` (IDE) | conforme : jette l'erreur (`\|_\|`), message propre « Numéro IDE suisse invalide (format ou checksum) » |
   | `routes/supplier_invoices.rs:404` (IBAN) | conforme : branche sur la variante, n'interpole pas le message |
   | `routes/journal_entries.rs:539, 542` ; `routes/opening_balances.rs:364, 367` ; `services/ecritures.rs:87, 99` ; `services/postes_ouverts.rs:67` | hors classe : l'erreur interpolée est celle de `Decimal`/`NaiveDate` (bibliothèque), non un `CoreError` qui porte déjà « invalide » |
   | `routes/onboarding.rs:163, 202, 378, 423` ; `routes/profile.rs:35` | hors classe : interpolent la **valeur saisie**, pas un message d'erreur |
   | `admin_backup/import.rs:281, 338` | hors classe : erreur `serde_json` / nom d'entrée, pas un `CoreError` |
   | `routes/supplier_invoices.rs:833`, `routes/bank_imports.rs:875`, `routes/admin.rs:296, 315`, `errors.rs:2689` | hors classe : le mot préfixé n'est pas celui du message interpolé |

   Les autres variantes de `CoreError` (`errors.rs:29-70`) ne sont préfixées par aucune route (le `grep` ne rend rien).
   Le tableau est l'inventaire clos : il a été établi par un `grep` plus large (`format!` sur `Validation`/`BadRequest`
   avec `{e}`, `{err}`, `{msg}`, `{reason}` dans `crates/kesh-api/src`), non par le seul motif cité ci-dessus, et chaque
   ligne de ce second relevé y est classée.
4. **Aucun test n'épingle la répétition.** `crates/kesh-api/tests/bank_accounts_e2e.rs:2131, 2135` n'exigent que
   `message.starts_with("IBAN invalide")` / `"QR-IBAN invalide"` : **vrais avant et après** le correctif (avec le helper
   QR-IBAN de l'AC-1, qui garde le nom du champ), donc muets contre la régression. `onboarding_path_b_e2e.rs:200-211`
   (IDE `"INVALID"`) ne lit que le code `VALIDATION_ERROR`. Les trois restent verts, inchangés, et ne suffisent pas : la story ajoute des assertions d'**égalité exacte**
   (AC-2).
5. **Le frontend ne figure pas le texte** : `grep -rn "IDE invalide" frontend/src frontend/tests` ne rend que
   `frontend/tests/e2e/contacts.spec.ts:69` (titre d'un test des contacts, route distincte, regex `/IDE|invalid/i` qui
   reste vraie). Le message est écrit en dur côté serveur (#710, hors périmètre).
   **Contrat de `CoreError`** (validation P1, P8) : `kesh-core/src/errors.rs:13` dit « Ne jamais exposer ces messages au
   frontend » ; or `kesh-api` sert ces trois variantes telles quelles (`AppError::Validation`). Le doc-comment est amendé
   (AC-1) : c'est un texte faux que la story épingle par des égalités exactes.

### #824

6. **Confirmée** — `docs/manual/fr/user-manual.tex:143-153`, § *Choix de la langue* : « Kesh gère le français,
   l'allemand, l'italien et l'anglais pour les messages du système. La langue est choisie lors de l'\textbf{onboarding}. »
   puis, dans un `keshnote` : « … dans la version actuelle, la langue est fixée à l'onboarding et n'est pas modifiable
   depuis un menu. »
7. **Ce que le code fait** : l'interface est servie dans la locale de l'installation —
   `routes/i18n.rs:20-22` (`let locale = &state.config.locale;`), remplie depuis `KESH_LANG` (`config.rs:845-848`,
   `env_nonempty("KESH_LANG").unwrap_or_else(|| "fr".into())`). L'étape 0 de l'assistant (`set_language`,
   `routes/onboarding.rs:155-175`) ne règle que `instance_language` de la société : la **langue des documents** — celle
   de la facture, de son PDF figé et de son e-mail d'envoi quand le contact n'a pas sa propre langue
   (`routes/invoice_email.rs:71`, `contact.language.unwrap_or(company.instance_language)` ; `routes/issued_invoice_pdf.rs:342`).
   **Et rien d'autre** (validation P2, P1) : le PDF d'un avoir suit `KESH_LANG` (`credit_notes.rs:346, 351, 357`), le courrier
   de réinitialisation du mot de passe aussi (`auth.rs:862`), les rapports `accounting_language` (`reports.rs:919`). Écart
   tracé en **#859** (bug), limite citée au manuel, au guide et au CHANGELOG ; le libellé « Langue des documents » reste
   (C-v017-6-9), sans aide de champ à *Paramètres* (aucune n'existe pour ce champ).
   Le frontend recharge les messages après le choix (`+page.svelte:44-47`, `loadI18nMessages`) mais le serveur les sert
   dans la locale de l'installation : le choix ne change pas l'interface.
8. **Le manuel d'administration le dit correctement à une ligne, et faux à une autre** : `admin-manual.tex:757`, tableau
   *Serveur \& réseau* — « KESH\_LANG & Locale de l'instance (fr/de/it/en). & fr » : exact mais **laconique** (ni « de
   l'interface », ni « l'assistant n'y touche pas »). **Et `admin-manual.tex:1410`**, § *Configuration de la company via
   l'onboarding* : « \textbf{Langue} de l'interface, puis \textbf{mode} » attribue la langue de l'interface à l'étape 0
   (validation P1, P2 ; C-v017-6-5 levé).
9. **Inventaire clos des textes qui attribuent la langue de l'INTERFACE à l'assistant** — la classe se cherche **par la
   valeur** (validation P1, P2), non par la formulation : `grep -rniE "langue.{0,20}interface|interface.{0,40}(onboarding|assistant)|(onboarding|assistant).{0,60}interface" docs README.md website`
   et les équivalents des trois autres langues (`Oberflächensprache`, `lingua dell.interfaccia`, `interface language`) dans
   `docs/`, `README.md`, `website/`, `crates/kesh-i18n/locales/`, `frontend/src` — plus l'ancienne recherche de formulation
   (`choisie.*langue`, `fixée`). **Dix** sites faux, sur **onze** fichiers distincts (quatre catalogues, le composant Svelte, deux manuels, le guide, le README, `company.rs`, `i18n.rs`) :

   | Site | Texte | Verdict |
   |---|---|---|
   | `user-manual.tex:146` | « La langue est choisie lors de l'onboarding. » | **faux** — à récrire |
   | `user-manual.tex:149-152` | note « fixée à l'onboarding et n'est pas modifiable depuis un menu » | **faux** — à récrire (le sélecteur reste à venir ; ce qui est faux, c'est l'origine) |
   | `user-manual.tex:3732` | limites : « Sélecteur de langue dans l'interface (la langue est fixée à l'onboarding … ) » | **faux** — à récrire, **non nommé par l'issue** |
   | `README.md:53` | « messages d'erreur API en FR/DE/IT/EN (langue choisie à l'onboarding ; sélecteur … à venir) » | **faux** — même défaut, **non nommé par l'issue** ; intégré (C-v017-6-3) |
   | `user-manual.tex:164-166` | `keshwarning` : « aux utilisateurs ayant choisi une autre langue » | **faux depuis la récriture** : l'utilisateur ne choisit pas la langue (P6) ; une phrase retouchée |
   | `admin-manual.tex:1410` | « Langue de l'interface, puis mode » (étape 0 de l'assistant) | **faux** — non nommé par l'issue (P2) |
   | `docs/user-guide/fr/getting-started.md:26` | « 1. Langue de l'interface (français, allemand, italien, anglais) » (étape 0) | **faux** — non nommé par l'issue (P2) |
   | `crates/kesh-db/src/entities/company.rs:164` ; `crates/kesh-api/src/routes/i18n.rs:3` | doc-comments : `instance_language` « Langue de l'interface utilisateur » ; `i18n.rs` « langue de l'instance » pour `KESH_LANG` | **faux** (validation P2, P6) : doc-comments seuls, retouchés (AC-4) |
| `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl:180` ; `frontend/src/routes/(app)/settings/+page.svelte:305` | `settings-field-instance-language` = « Langue de l'interface » / « Oberflächensprache » / « Lingua dell'interfaccia » / « Interface language » ; repli Svelte « Langue interface » ; l'écran *Paramètres* affiche `instanceLanguage` sous ce libellé | **faux — la racine du défaut** (P3) : c'est `instance_language`, la langue des documents |
   | `user-manual.tex:183-184, 201, 1140, 3392` ; `admin-manual.tex:1069, 1418` ; `messages.ftl:136` (`onboarding-accounting-lang-desc`, « découplée de la langue de l'interface ») et `user-manual.tex:149` (annonce du sélecteur) | « langue de l'installation », « langue comptable indépendante de l'interface », « langue de l'instance » | conformes |
   | `docs/kesh-specifications.txt:491, 1457, 1546, 2246, 2454, 2816` | spécification historique (FR76, UX-DR27 : langue interface = profil) | hors périmètre : document de conception, non un texte livré |
   | `website/*.html`, `marketing-brochure.tex`, `docs/manual/{de,en,it}/README.md` | aucune occurrence | sans objet |

   Les manuels DE/IT/EN ne sont que des `README.md` (« à traduire ») : aucune traduction du § à tenir. Les catalogues
   `de/it/en-CH` sont, eux, à tenir (libellé de *Paramètres*, AC-6).
10. **L'incise de la v017-2** : la même sous-section porte, dans le `keshwarning` (l. 156-163), « certains refus du
    serveur restent toutefois rédigés en français (\#710) ». **Elle est juste et ne bouge pas** : elle parle des refus de
    validation écrits en dur, indépendants de la locale. Le correctif de #832 ne la touche pas. (La phrase de l'étape des coordonnées
    de l'assistant, l. 202, troisième point de la liste du chemin B, qui cite le motif « qui peut être rédigé en français quelle que soit la langue de
    l'interface », reste également vraie.) Elle ne doit pas être **contredite** par la récriture : la phrase neuve dit
    que l'interface est dans la langue de l'installation, et la mise en garde dit que certains refus échappent à cette
    langue — les deux tiennent ensemble. **Mais la même `keshwarning` dit** « aux utilisateurs ayant choisi une autre
    langue » (l. 165-166) : fausse une fois la langue fixée par l'administrateur — retouchée (AC-5, P6).
11. **Vocabulaire** (validation P1, P4). Deux termes, chacun à un sens fixé : **« langue des documents »** désigne
    `instance_language` (réglée à l'assistant et dans *Paramètres*, héritée par les contacts) ; **« langue de l'interface »**
    désigne `KESH_LANG` (réglée par l'administrateur). Le manuel appelle aussi la première « langue de l'instance »
    (`user-manual.tex:1140, 1181-1182` ; clé `contact-form-language-inherited`) : ces passages ne sont pas rebaptisés.
    **« Instance » ne figure dans aucune des phrases récrites** (validation P2, P2) : `KESH_LANG` est « de l'installation »
    (`admin-manual.tex:757` compris), `instance_language` est « des documents ». « Documents » = la facture, son PDF et son
    e-mail d'envoi, rien de plus (limite : #859). Seuls les passages qui **mélangent** les termes sont récrits.

## Critères d'acceptation

### #832 — le refus qui se répète

- **AC-1 — Un seul préfixe, le nom du champ gardé.** Les trois sites IDE et IBAN (`onboarding.rs:538, 601` ;
  `bank_accounts.rs:320`) rendent `AppError::Validation(e.to_string())` : le message du `CoreError`, tel que `thiserror`
  le forme, sans second préfixe. Les deux sites QR-IBAN (`onboarding.rs:607`, `bank_accounts.rs:327`) passent par **un
  helper unique** (DRY) de `kesh-api`, à placer dans `crates/kesh-api/src/util.rs` : `CoreError::InvalidIban(m)` (erreur
  d'IBAN propagée par `QrIban::new`) → `format!("QR-IBAN invalide : {m}")` ; toute autre variante → `e.to_string()`
  (elle porte déjà « QR-IBAN invalide »). **Même classe, intégrée** (validation P2, P9) : `qr_iban.rs:42-43` écrit
  « QR-IBAN doit être suisse ou liechtensteinois », ce qui servait « QR-IBAN invalide : QR-IBAN doit être … » ; le message
  de `kesh-core` devient « doit être suisse ou liechtensteinois » (un test unitaire de `kesh-core` fige
  `err.to_string() == "QR-IBAN invalide : doit être suisse ou liechtensteinois"`, mutation M6 : remettre l'ancien texte). Le champ désigné reste le bon, le `starts_with("QR-IBAN invalide")` existant
  (`bank_accounts_e2e.rs:2135`) reste vrai. Le doc-comment de `CoreError` (`errors.rs:13`) est amendé : ces messages sont
  servis tels quels par `kesh-api` (#710), la phrase « Ne jamais exposer … » est retirée ou nuancée. Le doc-comment du site de l'assistant
  (`onboarding.rs:535`, « Validate IDE via kesh-core if provided ») dit que le message de `kesh-core` porte déjà le
  nom du champ. Le motif lu par l'utilisateur devient « Numéro IDE invalide : 9 chiffres attendus après CHE, trouvé 3 »,
  « IBAN invalide : trop court », « QR-IBAN invalide : trop court » (QR-IBAN mal formé), « QR-IBAN invalide : QR-IID 12345 hors plage 30000-31999 ».
- **AC-2 — Le message est figé, à l'égalité exacte.** Quatre assertions, **tests rouges d'abord** :
  (a) `onboarding_path_b_e2e.rs`, le test de l'IDE invalide (`:200-211`) : `body["error"]["message"]` **égal** à
  « Numéro IDE invalide : doit commencer par CHE » (entrée `"INVALID"` : `che_number.rs:83`) ;
  (b) un test de l'IBAN invalide de l'étape banque de l'assistant (`POST /api/v1/onboarding/bank-account`, IBAN `"BAD"`) :
  message **égal** à « IBAN invalide : trop court » (`iban.rs:148`, longueur < 5) — ou la valeur que le développeur relève
  au rouge, **recopiée du serveur d'avant correction moins le préfixe**, jamais devinée ;
  (c) le même pour le QR-IBAN de l'assistant (IBAN valide, QR-IBAN `"BAD"`) : message **égal** à « QR-IBAN invalide :
  trop court » (l'erreur d'`Iban::new`, étiquetée par le helper) — c'est ce test qui fige que le champ désigné est le QR-IBAN ;
  un second cas, QR-IBAN d'IBAN valide hors de la plage QR-IID (message « QR-IBAN invalide : QR-IID … hors plage … »), fige
  la branche « toute autre variante » ;
  (d) `bank_accounts_e2e.rs`, `ordre_des_refus_de_forme` (`:2111-2147`) : en plus du `starts_with`, une assertion
  que `message.matches("IBAN invalide").count() == 1` pour les deux cas IBAN/QR-IBAN (« QR-IBAN invalide » ne contient
  « IBAN invalide » qu'**une** fois ; le message neuf n'en a qu'un, l'ancien « QR-IBAN invalide : IBAN invalide : trop court »
  en avait deux). Le `starts_with` existant
  reste, inchangé.
  Les entrées sont des fixtures synthétiques ; aucune donnée réelle.
- **AC-3 — Mutations (sous le verrou de gate, un site à la fois, tuées une à une).** M1 : remettre
  `format!("IDE invalide : {e}")` à `onboarding.rs:538` → (a) rougit. M2 : remettre `format!("IBAN invalide : {e}")` à
  `onboarding.rs:601` → (b) rougit. M3 : remettre `format!("QR-IBAN invalide : {e}")` à `onboarding.rs:607` → (c) rougit. M3-bis : rendre au site QR-IBAN le message nu `e.to_string()` (« IBAN invalide : trop court ») → (c) et (d) rougissent. M4 : idem `bank_accounts.rs:320` → (d) rougit. M5 : idem
  `bank_accounts.rs:327` → (d) rougit. Le développeur consigne pour chacune le test qui rougit ; après chaque
  restauration, **toucher le fichier** (binaire périmé, mémoire `mutation-restauree-binaire-perime`).
- **AC-4 — Aucun autre site de code serveur ne change.** `git diff --stat`, côté code serveur : `routes/onboarding.rs`,
  `routes/bank_accounts.rs`, `util.rs`, `kesh-core/src/errors.rs` (doc-comment) et `kesh-core/src/types/qr_iban.rs` (message et son test), les doc-comments de
  `kesh-db/src/entities/company.rs:164` (« Langue des documents : celle de la facture, de son PDF et de son e-mail d'envoi
  quand le contact n'a pas la sienne ») et de `routes/i18n.rs:3` (« langue de l'installation ») et les deux fichiers de test seulement. Les messages de `contacts.rs:322` et des sites « hors classe » (§ 3) sont
  intouchés.

### #824 — le manuel

- **AC-5 — Le § *Choix de la langue* est récrit** (`user-manual.tex:143-168`), avec le vocabulaire de § 11 (« langue de
  l'interface » = `KESH_LANG` ; « langue des documents » = `instance_language`). Il dit, sans rien promettre de plus :
  (i) l'interface est servie dans **la langue de l'installation**, réglée par l'administrateur (`KESH_LANG`, `fr` par
  défaut ; manuel d'administration, tableau des variables d'environnement) ; (ii) l'assistant d'onboarding, à son premier
  écran (choix de la langue), ne règle **que la langue des documents** — celle des factures, de leur PDF et de leur e-mail d'envoi,
  quand le contact n'a pas sa propre langue —, non celle de l'interface ; « instance » n'y figure pas (§ 11),
  ni « PDF » ou « courriers » sans qualificatif, et la limite est citée : le PDF d'un avoir et le courrier de réinitialisation suivent la langue de
  l'installation, #859 ; (iii) la langue de l'interface ne se change pas depuis un menu, et un sélecteur reste prévu. Texte
  prescrit (le développeur peut l'ajuster sans en changer le sens) :
  > Kesh gère le français, l'allemand, l'italien et l'anglais pour les messages du système. L'interface s'affiche dans
  > la \textbf{langue de l'installation}, que fixe l'administrateur (variable \texttt{KESH\_LANG}, français par défaut ;
  > voir le manuel d'administration). Le tout premier écran de l'assistant d'onboarding, avant le choix du mode, ne règle pas
  > cette langue : il règle celle des \textbf{documents}, c'est-à-dire des factures, de leur PDF et de leur e-mail d'envoi,
  > employée quand un contact n'a pas sa propre langue. Le PDF d'un avoir et le courrier de réinitialisation du mot de
  > passe suivent, eux, la langue de l'installation (\#859).
  >
  > `keshnote` : Un sélecteur permettant de changer la langue de l'interface depuis un menu est prévu pour une version
  > ultérieure : dans la version actuelle, elle ne se change pas depuis l'application.

  **La `keshwarning` qui suit** (traductions non relues, #710, « avant la 0.11.0 ») est conservée, **sauf une phrase**
  (validation P1, P6) : « aux utilisateurs ayant choisi une autre langue » (`:165-166`) devient « aux installations
  réglées dans une autre langue » — l'utilisateur ne choisit pas la langue. Le reste est inchangé, y compris l'incise
  #710.
- **AC-6 — Les autres sites suivent** (inventaire § 9, **huit** sites faux dont le § ci-dessus) :
  - `user-manual.tex:3732` devient « Sélecteur de langue dans l'interface (la langue de l'interface est celle de
    l'installation, \texttt{KESH\_LANG} ; le sélecteur est prévu pour une version ultérieure). » ;
  - `README.md:53` devient « **Multilingue** — interface et messages en FR/DE/IT/EN (langue de l'installation,
    `KESH_LANG` ; certains refus du serveur restent en français, #710 ; sélecteur de langue dans l'interface à venir) »
    (P7) ;
  - `admin-manual.tex:1410` devient « \textbf{Langue des documents} (la langue de l'interface est celle de
    l'installation, \texttt{KESH\_LANG}), puis \textbf{mode} (guidé ou expert). » ; `admin-manual.tex:757`, « Locale de
    l'instance » devient « Langue de l'interface (locale de l'installation) » (P4, P2-P2) ; PDF d'administration régénéré
    (`make admin`) ;
  - `docs/user-guide/fr/getting-started.md:26` devient « 1. **Langue des documents** (français, allemand, italien,
    anglais) --- celle des factures, de leur PDF et de leur e-mail d'envoi ; la langue de l'interface est celle de
    l'installation (`KESH_LANG`) ; le PDF d'un avoir et le courrier de réinitialisation suivent cette dernière (#859). » ;
  - **l'écran *Paramètres*** (P3) : `settings-field-instance-language` se lit « **Langue des documents** » (fr-CH),
    « Dokumentensprache » (de-CH), « Lingua dei documenti » (it-CH), « Document language » (en-CH), aux lignes 180 des
    quatre `messages.ftl`, et le repli Svelte (`settings/+page.svelte:305`) devient `'Langue des documents'` — égal à la
    valeur fr-CH. Contrôles : `npm run test:unit` (gardes `i18n-repli-divergent-actif`, `i18n-un-repli-par-cle`,
    `i18n-keys`, `i18n-libelle-en-dur`), `npm run lint-i18n-ownership`, test de parité des quatre catalogues (nombre de
    clés inchangé : une valeur change, aucune clé ne naît).
- **AC-7 — Le contrôle des manuels se fait sur le PDF, aux deux bornes** (P5). Le PDF porte l'apostrophe typographique
  `’` : un fragment écrit avec `'` rend zéro **avant** comme après. Après `make user` et `make admin`, aplatir
  (`pdftotext <pdf> - | tr '\n' ' ' | tr -s ' '`) et compter **avant** (PDF du commit de départ, `git show
  8e126cfd:docs/manual/fr/user-manual.pdf | pdftotext - - | …`, jamais `HEAD`) et **après**, par `grep -oF "<fragment>" | wc -l`, les deux
  graphies (`’` et `'`) de chaque fragment : `fixée à l’onboarding` (avant ≥ 2, après 0), `choisie lors de l’onboarding`
  (avant ≥ 1, après 0) et `ayant choisi une autre langue` (avant 1, après 0) pour le manuel utilisateur ; `Langue de l’interface, puis` et
  `Locale de l’instance` (avant 1 chacun, après 0) pour le manuel d'administration ; et les phrases neuves présentes
  (après ≥ 1) : `langue de l’installation, que fixe` (utilisateur) et `locale de l’installation` (administration). **Attendu : avant > 0, après = 0** — un zéro aux deux
  bornes est un contrôle muet. Recopier les décomptes au Dev Agent Record. La macro `\keshVersion` n'est **pas**
  bumpée. Les `.tex`, PDF, `README.md` et guide voyagent dans le même commit que leur motif.
- **AC-8 — Grep de la valeur, pas de la formulation** (*Propagation post-patch*). Après la récriture, les deux relevés de
  § 9 (`grep -rniE "langue.{0,20}interface|interface.{0,40}(onboarding|assistant)|(onboarding|assistant).{0,60}interface" docs README.md website`
  et son équivalent sur `crates/kesh-i18n/locales` et `frontend/src`, plus `(choisie|choisi).{0,40}langue|langue.{0,40}(choisie|fixée)` sur `docs README.md`, qui retrouve la ligne 146 du manuel)
  sont rejoués ; chaque ligne restante est triée à la main au Dev Agent Record (attendues : les « conformes » du
  tableau § 9, et la spécification historique).

### Les deux

- **AC-9 — CHANGELOG.** `CHANGELOG.md`, `## [0.17.0] — Non publié`, section *Corrigé* : deux puces. La première :
  « **Un refus d'IDE, d'IBAN ou de QR-IBAN ne répète plus son mot** ([#832](…)) : « Numéro IDE invalide : … » et non
  « IDE invalide : Numéro IDE invalide : … », à l'assistant d'installation comme à la création d'un compte bancaire. »
  La seconde : « **Les textes disent d'où vient la langue** ([#824](…)) : la langue de l'interface est celle de
  l'installation (`KESH_LANG`) ; l'assistant ne règle que la langue des documents — celle des factures, de leur PDF et de leur e-mail d'envoi —, ce que
  l'écran *Paramètres*, les manuels et le guide de démarrage affichent désormais ; le PDF d'un avoir et le courrier de
  réinitialisation suivent encore la langue de l'installation ([#859](…)). » Liens comme aux puces voisines. L'intro « La 0.17.0 réunit six
  stories » (l. 13) est **laissée** : son décompte appartient à la PR de release.
- **AC-10 — Aucun mot-clé de fermeture** dans les messages de commit (`refs #832`, `refs #824` seulement ; contrôle par
  `git log 8e126cfd..HEAD --format=%B | grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'` vide).
- **AC-11 — Le gate** (le frontend est touché : un libellé et un repli). `fmt` + `clippy` workspace, puis **gate backend
  complet** (`scripts/test-fast.sh` sous le verrou, base remise à zéro d'abord) ; côté frontend, `npm run check`,
  `npm run lint-i18n-ownership`, `npm run test:unit`, `npm run build`, puis **l'E2E complet au dernier commit de code**
  (7 échecs KF-029 attendus, `docs/testing.md`). **Recette visuelle** (thème clair, base de démonstration, hors dépôt
  sous `kesh-gate-logs/`) : une capture de l'écran *Paramètres* en `fr-CH`, une autre en `de-CH` (`KESH_LANG=de`), le
  libellé « Langue des documents » / « Dokumentensprache » lisible. Le Dev Agent Record n'affirme que ce qui a tourné.

## Tâches

1. Tests rouges d'abord (AC-2), commit « tests rouges ». 2. Correctif des cinq sites, helper QR-IBAN et message de `qr_iban.rs` (AC-1), mutations
M1 à M5, M3-bis et M6 (AC-3). 3. Libellé de *Paramètres*, quatre locales, repli (AC-6). 4. Manuels `.tex`, PDF, `README.md`,
guide (AC-5 à AC-8). 5. CHANGELOG (AC-9). 6. Gate complet et E2E (AC-11), captures, Dev Agent Record, `sprint-status.yaml`.

## Dérogation règle de splitting

Aucune : `kesh-api` (+ `kesh-core` pour un doc-comment), le frontend (un libellé), `kesh-i18n` (quatre valeurs) et la
documentation — quatre modules au plus, tous mécaniques ; le critère « plus de cinq modules » n'est pas atteint.

## Issues à ouvrir ou à commenter

- **#859** (déjà ouverte) : citée comme limite (AC-5, guide, CHANGELOG) ; rien à poser.
- **#832** et **#824** : le complément de la fiche a **déjà été publié** en commentaire le 2026-10-31 (validation P1, P9) ;
  l'orchestrateur le met à jour si le périmètre élargi (QR-IBAN par helper ; huit sites faux dont *Paramètres*, le
  manuel d'administration et le guide) doit y figurer. Rien à poser par la fiche.
- Aucune issue neuve : le message reste en français en dur (#710, déjà ouverte) ; le libellé de *Paramètres* est intégré
  (C-v017-6-9), non tracé à part.

## Change Log

- 2026-10-31 — fiche créée (Sonnet 5.5, autonomie). Prémisses de #832 et #824 établies au code ; deux extensions
  mécaniques (IBAN/QR-IBAN ; `user-manual.tex:3732` et `README.md:53`). Prochaine : validation P1 (Opus, une lentille
  complète R+F, `v017-6-validate-prompt-p1.md`).
- 2026-10-31 — **Validation P1** (Opus, une lentille R+F, `v0176-validate-p1.md`) : 0 CRITICAL, 2 HIGH, 4 MEDIUM, 6 LOW,
  tous remédiés. P1 (HIGH) : `QrIban::new` propage l'erreur `InvalidIban` ; `e.to_string()` aurait rendu un refus de
  QR-IBAN désignant l'IBAN et fait rougir `bank_accounts_e2e.rs:2135` — helper commun aux deux sites QR-IBAN (AC-1,
  AC-2 c, M3-bis). P2 (HIGH) : inventaire refait par la valeur, huit sites faux (dont `admin-manual.tex:1410` et
  `getting-started.md:26`), C-v017-6-5 levé. P3 : libellé de *Paramètres* intégré (C-v017-6-9). P4 : vocabulaire fixé (§ 11).
  P5 : contrôle du PDF aux deux bornes, deux graphies d'apostrophe. P6 : une phrase de la `keshwarning` retouchée.
  P7 à P12 : README, doc-comment de `CoreError`, section des commentaires d'issue, motif de grep réel + `import.rs:281,
  338`, « assistant d'onboarding », AC-4 « côté code ». Prochaine : validation P2 (Sonnet, une lentille R+F,
  `v017-6-validate-prompt-p2.md`).
- 2026-10-31 — **Validation P2** (Sonnet, une lentille R+F, `v0176-validate-p2.md`) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 8 LOW,
  tous remédiés. P1 (MEDIUM) : « langue des documents » bornée à la facture, son PDF et son e-mail d'envoi (AC-5, guide,
  CHANGELOG), limite #859. P2 (MEDIUM) : « installation » pour `KESH_LANG` (`admin-manual.tex:757` compris), « instance »
  retiré des phrases récrites. P3 : motif d'AC-8. P4 : plage `8e126cfd..HEAD`. P5 : onze fichiers (après l'ajout de P6).
  P6 : `company.rs:164` et `i18n.rs:3` à l'inventaire et à AC-4. P7 : fragments d'AC-7 complétés, référence `8e126cfd`.
  P8 : C-v017-6-1 et -3 annotés. P9 : message de `qr_iban.rs` intégré (M6). P10 : repères. Prochaine : validation P3
  (Opus, ciblée sur cette remédiation, `v017-6-validate-prompt-p3.md`).

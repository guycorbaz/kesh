# Story 15-14a : Manuels et libellés — dix défauts P3/P4 qui disent autre chose que le code

Status: review

<!-- Spécifiée le 2026-10-09 sur origin/main = dc4bc58b (worktree kesh-15-14). Sous-story de la 15-14
     (lot de défauts de documentation et de libellés, découpé d'emblée — C-15-14-1). Choix C-15-14-1 à 8.
     Rebasée sur bcded0c8 (15-13a mergée) à la validation P1 : numéros de ligne réalignés sur ce commit ;
     choix C-15-14-11 à 16. Validation P2 : inventaires réécrits en commandes comptées (C-15-14-17),
     choix C-15-14-17 à 20, 23 ; dépendance envers la 15-7b1 (PR #583).
     Validation P3 : branche rebasée sur origin/main = 245b91ee (15-7b1 mergée, dépendance satisfaite) ;
     inventaires sur tout le dépôt suivi, exclusions justifiées ; G13 ; choix C-15-14-25 à 27, 31.
     Validation P4 : 0 MEDIUM sur cette fiche — validation CLOSE (C-15-14-32) ; LOW appliqués. -->

## Story

En tant qu'utilisateur de Kesh qui suit le manuel ou lit un message d'erreur,
je veux que le texte décrive ce que le logiciel fait réellement — taux, menus, plans comptables,
imports, versions, ordre de réouverture, vocabulaire —,
afin de ne pas chercher un écran qui n'existe pas ni tenter un geste que Kesh refusera.

**Issues fermées** (une ligne `closes` par issue dans le titre ou le corps de la PR — § *Commits qui
adressent une issue* du `CLAUDE.md`) :

| Issue | Prio. | Labels | Objet | État au code (`bcded0c8`) |
|---|---|---|---|---|
| #539 | P3 | bug, documentation | taux de TVA du manuel (2,5 / 3,7 %) | **non corrigé** — `user-manual.tex:875`, `:1928` |
| #547 | P4 | bug | « Réglages » au lieu de « Paramètres » | **non corrigé** — 4 sites au manuel **+ 2 messages + 1 commentaire de `.env.example` + `README.md:34`** (propagation) |
| #488 | P3 | bug, documentation | « deux plans pré-configurés (Sterchi PME, KMU) » | **non corrigé** — 14 lignes dans 4 documents (inventaire de l'AC 3) |
| #291 | P3 | documentation | « Contacts → Import CSV » inexistant | **non corrigé** — `user-manual.tex:857-859` |
| #458 | P3 | documentation | « dossier surveillé » | **non corrigé** — 4 sites (+2 historiques assumés) |
| #449 | P3 | bug | faille KF-036 annoncée ouverte | **non corrigé** — `api-external.md:484` **+ 2 sites au manuel d'administration** |
| #432 | P4 | bug | `[#NNN]` du README | **non corrigé** — 54 références, lignes 211-223 |
| #569 | P3 | bug | « rouvrir l'exercice » sans l'ordre LIFO | **non corrigé** — 6 clés ×4, 6 replis Rust, 4 replis frontend, 5 phrases du manuel, 1 ligne d'API |
| #321 | P4 | bug | `MwSt` contre `MWST` (de-CH) | **non corrigé** — 5 valeurs + 1 commentaire |
| #323 | P3 | known-failure | « Clôturer » = « Fermer » dans les cibles | **non corrigé** — 2 clés ×3 + titre/corps/lien de-CH + 6 participes de-CH (inventaire de l'AC 10) |

`refs #459` (automatisation de l'import, que #458 renvoie). **Huit** de ces issues portent `bug` ou
`known-failure` et comptent dans le décompte de l'engagement 1 (#539, #547, #488, #449, #432, #569,
#321, #323) ; #291 et #458 ne portent que `documentation`.

**Hors de cette story** (C-15-14-2) : #579 (15-6c), #324 (prémisse fausse, arbitrage de Guy), #469,
#339, #504 ; et la 15-14b (#575, #554, #127), qui suit celle-ci.

## Acceptance Criteria

> Convention : « relevé sur `bcded0c8` » (réalignement de la validation P1 ; les manuels utilisateur, la
> brochure hors `:542` et les catalogues n'ont pas bougé depuis `dc4bc58b`, le manuel d'administration et
> `.env.example` si) — les numéros de ligne **se périment** ; le développeur
> retrouve chaque site **par la valeur** (commande donnée), jamais par le numéro. Pour chaque
> manuel modifié : **régénérer le PDF** (`scripts/mem-guard.sh make -B -C docs/manual fr`) et le
> **contrôler aplati et normalisé** (validation P2, F-1). Le PDF aplati n'a **plus d'apostrophe
> droite** (`'` y devient `’`), **perd les traits d'union** coupés en fin de ligne
> (`super-administrateur` → `superadministrateur`, `QR-facture` → `QRfacture`) et porte des ligatures
> et des espaces insécables : un `grep -F "l'exercice"` sur `pdftotext` brut rend **0** sur un texte
> présent — tout contrôle « ancien absent » y passerait **vert avant correction**. Fonction de
> contrôle, à coller telle quelle (elle normalise le PDF **et** le motif de la même façon ; espaces
> et traits d'union sont retirés des deux côtés) :
>
> ```sh
> export LC_ALL=C.UTF-8   # obligatoire : sous LC_ALL=C, la classe [’‘ʼ] est lue octet par octet et
>                         # tout compte tombe à 0 (validation P3, L-3 = F-3) ; mem-guard.sh ne propage
>                         # pas forcément LANG
> norm() { sed "s/[’‘ʼ]/'/g; s/ﬁ/fi/g; s/ﬂ/fl/g; s/ﬀ/ff/g; s/ﬃ/ffi/g; s/ﬄ/ffl/g; s/\xc2\xad//g; s/\xc2\xa0/ /g; s/\xe2\x80\xaf/ /g; s/\xe2\x80\x89/ /g" | tr -d '\n -'; }
> occ() { pdftotext "$1" - | norm > "${TMPDIR:-/tmp}/aplati"; printf '%s' "$2" | norm > "${TMPDIR:-/tmp}/motif"; grep -oF -f "${TMPDIR:-/tmp}/motif" "${TMPDIR:-/tmp}/aplati" | wc -l; }
> occ docs/manual/fr/user-manual.pdf "doit d'abord le rouvrir"     # 3 sur bcded0c8
> ```
>
> Le motif s'écrit **tel que rendu** (« --- » → « — », `\%` → `%`, sans commande LaTeX). **Chaque
> contrôle « ancien absent » est précédé du même `occ` sur le PDF d'avant** (`git show
> bcded0c8:docs/manual/fr/<f>.pdf > "${TMPDIR:-/tmp}/avant.pdf"`), **qui doit rendre ≥ 1** : c'est la preuve
> que le motif sait trouver le texte. Vérifié à la validation P2 sur les PDF de `bcded0c8` :
> `doit d'abord le rouvrir` → 3, `un administrateur doit d'abord rouvrir l'exercice` → 1,
> `créer le compte super-administrateur` → 1, `n'est pas dans la v0.9.0` → 1 (admin), `QR-facture`
> → 6, `Sterchi PME` → 6 (user) et 1 (admin), `dossier surveillé` → 1, `2.5%` → 2. **Rejoué en validation
> P3 sur les PDF de `245b91ee`** (régénérés par la 15-7b1) : mêmes valeurs, sauf `Sterchi PME` (user) → 5
> (le site `:182`, corrigé par la 15-7b1) et `un administrateur doit d'abord rouvrir l'exercice` → **2** —
> le PDF de `bcded0c8` était **en retard sur son `.tex`** (le site `:1208-1209` n'y figurait pas). D'où :
> au T0, **régénérer les PDF de l'état de départ avant** le contrôle « présent avant », pour qu'il porte
> sur le texte réel. Limite résiduelle : une phrase
> coupée par un saut de page (en-tête ou pied intercalé) échappe à `occ` — faux 0 possible sur un contrôle
> « ancien absent » ; G12 et les gardes sur le `.tex` couvrent ce risque pour #569, pas pour les autres AC
> (validation P3, F-3 : angle mort déclaré).
>
> **Inventaires** (validation P2 — trois MEDIUM y sont nés de la P1, tous des inventaires déclarés
> complets qui ne l'étaient pas ; validation P3 — deux MEDIUM nés d'un **périmètre de commande** trop
> étroit). Chaque AC qui corrige une famille de textes donne la **commande** d'inventaire, son **résultat
> compté**, et la **partition** corrigé / assumé-avec-raison, de sorte que le compte se recalcule. Au T0,
> le développeur relance chaque commande : un compte qui diffère se ventile avant toute écriture — jamais
> « tout autre site s'ajoute » sans recompte.
>
> **Périmètre : tout le dépôt suivi, et c'est l'exclusion qui se justifie** (validation P3, F-1, R-3 ;
> C-15-14-25). Chaque commande est un `git grep -I` sur `.` (fichiers suivis, binaires écartés — les PDF se
> contrôlent par `occ`), moins l'ensemble d'exclusions commun `E`, écrit une fois ici avec sa raison :
>
> ```sh
> export LC_ALL=C.UTF-8
> E=(':(exclude)_bmad-output' ':(exclude)_bmad' ':(exclude).claude' ':(exclude)CLAUDE.md' ':(exclude)CHANGELOG.md' ':(exclude).svelte-kit' ':(exclude,glob)docs/kesh-specifications.*' ':(exclude)docs/kesh-prd-v0.2.md' ':(exclude)docs/change_request.md' ':(exclude)docs/known-failures.md' ':(exclude,glob)**/*.test.ts' ':(exclude,glob)**/*.test.svelte' ':(exclude)frontend/tests' ':(exclude,glob)crates/*/tests/**')
> ```
>
> | Exclu | Raison |
> |---|---|
> | `_bmad-output/` | artefacts de planification (fiches, registre, sprint-status) : ils citent les anciens textes par nécessité |
> | `_bmad/`, `.claude/` | cadre BMAD et skills installés : tiers, hors produit |
> | `CLAUDE.md` | instructions de travail (affaire de Guy, #577) |
> | `CHANGELOG.md` | notes de versions publiées : on ne réécrit pas l'historique ; l'entrée de cette story s'y ajoute (AC 11) |
> | `.svelte-kit/` | fichiers engendrés par SvelteKit (versionnés à la racine du dépôt) |
> | `docs/kesh-specifications.*`, `docs/kesh-prd-v0.2.md` | spécification et PRD de conception, datés (validation P3, F-7) |
> | `docs/change_request.md`, `docs/known-failures.md` | archivés (`CLAUDE.md` § *Legacy*) |
> | `**/*.test.ts`, `**/*.test.svelte`, `frontend/tests/`, `crates/*/tests/` | tests : ils n'affichent rien ; un test qui fige un ancien texte rougit au gate, et l'AC 8 inventorie à part les Vitest qui le figent |
>
> Les comptes ci-dessous sont ceux de **`245b91ee`** (15-7b1 mergée ; validation P3), sauf mention. Les
> **numéros de ligne** restent ceux de `bcded0c8` (C-15-14-25) ; sur `245b91ee`, la 15-7b1 a décalé
> `user-manual.tex` de **+1 à partir de la ligne 190** et de **+15 à partir de la ligne 2243**,
> `CHANGELOG.md` de +1 à partir de la ligne 42, `vat_rates.rs` de +1 à partir de la ligne 353 ; le manuel
> d'administration et les catalogues sont inchangés aux lignes citées. Les deux lignes neuves de la
> 15-7b1 se citent avec leur numéro sur `245b91ee`, signalé comme tel. Dans une partition, une ligne de
> **code** est classée « commentaire » quand elle commence (blancs ôtés) par `//`, `*`, `/*`, `<!--`,
> `--` ou `#` — détecteur `grep -E '^[^:]+:[0-9]+:\s*(//|\*|/\*|<!--|--|#)'` — ou continue un commentaire
> multiligne (nommée alors une à une) ; ce détecteur ne s'applique **pas** aux fichiers de texte
> (`.tex`, `.md`, `.ftl`, `.env.example`), où `---` ou `#` sont du contenu.

### AC 1 — Les taux de TVA du manuel sont ceux que Kesh pose (#539)

Kesh pose 0 %, 2,6 %, 3,8 % et 8,1 % (`DEFAULT_SWISS_RATES`, `crates/kesh-db/src/repositories/vat_rates.rs:281`,
`valid_from = 2024-01-01`).

- `user-manual.tex:875` devient :
  `\item \textbf{Taux TVA} : 0\%, 2.6\%, 3.8\%, 8.1\% --- les taux suisses en vigueur depuis le 1\textsuperscript{er} janvier 2024, que Kesh pose à la création de la société (\emph{Paramètres} → \emph{Taux de TVA}).`
  (« au moment de la rédaction » disparaît : c'est lui qui a laissé la phrase se périmer.)
- `user-manual.tex:1928` : « ventilée par taux (8.1\%, 3.8\%, 2.6\%, 0\%) ».
- Inventaire : `git grep -InE '2[.,]5 ?\\?%|3[.,]7 ?\\?%|7[.,]7 ?\\?%' -- . "${E[@]}"`
  → **4** sur `245b91ee`. **Corrigés : 2** (`user-manual.tex:875`, `:1928`). **Assumés : 2** — le
  commentaire de la migration `20260613000001_vat_rates_crud.sql:11` (une migration appliquée ne se
  modifie plus, `CLAUDE.md` P8) et le doc-comment `crates/kesh-qrbill/src/types.rs:195` (« e.g. 7.70 for
  7.7% », exemple de format, non un taux en vigueur). 2 + 2 = 4 ; **2** après correction.
- **Test** (G1) : `le_manuel_cite_les_taux_poses_par_kesh`, dans un **`#[cfg(test)] mod tests` neuf** de
  `vat_rates.rs` (le fichier n'en a pas : `grep -n 'cfg(test)' crates/kesh-db/src/repositories/vat_rates.rs`
  ne rend rien ; précédents dans le même répertoire : `accounts.rs:1136`, `bank_profiles.rs:343`), test
  `#[test]` pur — **pas** `#[sqlx::test]`, que `test_schema_guard.rs` recense ; lieu et rayon de gate :
  C-15-14-13 — lit
  `docs/manual/fr/user-manual.tex` (`env!("CARGO_MANIFEST_DIR")/../../docs/…`) ; pour chaque taux de
  `DEFAULT_SWISS_RATES`, sa forme `{entier}.{décimale}\%` (`810` → `8.1\%`, `0` → `0\%`) figure sur la
  ligne `\textbf{Taux TVA}` ; aucune des formes `2.5\%`, `3.7\%`, `7.7\%` n'apparaît dans un
  `docs/manual/fr/*.tex`. Mutations : réécrire `3.7\%` au manuel → rouge ; changer `380` en `370` dans
  la constante → rouge (sur le manuel corrigé ; cette mutation touche une constante de production —
  restaurer par `git checkout` **puis `touch`**).

### AC 2 — Le manuel et les messages renvoient à « Paramètres » (#547, C-15-14-5)

Le menu affiche `nav-settings = Paramètres` ; l'écran de facturation s'intitule `Paramètres — Facturation`.

- `user-manual.tex:575` : « Dans \emph{Paramètres} $\rightarrow$ \emph{Verrou de période} » ;
  `:907` : « (\emph{Paramètres} → \emph{Facturation}) » ; `:952` : « \emph{Paramètres} $\rightarrow$
  section \emph{Organisation} » ; `:2085` : « \textbf{Paramètres et traçabilité} ».
- **Message** (propagation, même symptôme) : `invoice-default-revenue-account-unusable`, dont le texte
  renvoie aux « Réglages ». Texte attendu, la fin seule change :
  - fr-CH : « … — corrigez-le dans Paramètres → Facturation, ou choisissez un compte sur chaque ligne. »
  - de-CH : « … — korrigieren Sie es unter Einstellungen → Fakturierung oder wählen Sie pro Zeile ein Konto. »
  - it-CH : « … — correggilo in Impostazioni → Fatturazione oppure scegli un conto per ogni riga. »
  - en-CH : « … — fix it in Settings → Invoicing, or pick an account on each line. »
  - repli `frontend/src/lib/components/invoices/InvoiceForm.svelte:768` **identique** au catalogue fr-CH,
    gardé par **G13** (validation P3, R-1 : `i18n-repli-divergent-actif.test.ts` ne garde **pas** une clé à
    un seul site d'appel — il ne retient que les clés à au moins deux replis distincts, `parTexte.size > 1`).
  - Les trois locales autres que fr-CH disent **déjà** le menu (« Einstellungen », « Impostazioni »,
    « Settings ») : leur réécriture en « → Fakturierung » etc. est une **extension de périmètre** voulue,
    que G3 impose pour que les quatre valeurs nomment le même écran (validation P3, L-7).
  - **Hors périmètre, tracé** (validation P4, L-2 de la lentille R) : « Paramètres → Facturation » est le
    **titre** de l'écran (`settings-invoicing-title`), non un chemin de menu — l'entrée « Facturation » est
    une **sœur** de « Paramètres » dans le groupe *Administration* (`frontend/src/routes/(app)/+layout.svelte:143`),
    et la page `/settings` n'a aucun lien vers `/settings/invoicing`. La convention est déjà écrite une
    douzaine de fois au manuel ; cette story ne la change pas. Issue **#585** (P4), ouverte par
    l'orchestrateur le 2026-10-09 ; ni `closes` ni `refs` ici.
- **Second message** (validation P1, R3 = F-11 ; C-15-14-11) : fr-CH `error-invoice-pdf-header-overflow`
  (`fr-CH:1887`) dit « Supprimez une coordonnée … **dans les réglages** : … » — un renvoi à l'écran où l'on
  agit, que les trois autres locales nomment déjà par le menu (« in den Einstellungen », « nelle
  impostazioni », « in the settings »). fr-CH : « … dans les Paramètres : les raccourcir … » (seul ce
  segment change) ; repli Rust `crates/kesh-api/src/errors.rs:1807` (chaîne sur plusieurs lignes, `\` de
  continuation) égal au catalogue ; aucun repli frontend (`pdf-error.ts:21` ne porte que la clé).
- **Commentaire de `.env.example:305`** : « la société s'il est renseigné (Réglages) » → « (Paramètres) ».
- **`README.md:34`** (validation P3, F-1 — hors de la commande (B) d'alors) : « saisis une fois dans les
  réglages » désigne l'écran où l'on saisit les coordonnées — un renvoi à l'écran où l'on agit, au sens de
  C-15-14-11 → « saisis une fois dans les Paramètres ».
- **Inventaire, deux commandes, la casse comptant**, sur tout le dépôt suivi moins `E` (validation P3,
  F-1 ; C-15-14-25 — la P2 bornait (B) à quatre fichiers, et `README.md:34` lui échappait). Unité : la
  **ligne** (une ligne peut porter plusieurs occurrences, `user-manual.tex:380` en porte quatre).
  - (A) `git grep -In 'Réglages' -- . "${E[@]}"` → **21** lignes sur `245b91ee`. **Corrigées : 7** —
    `user-manual.tex:575`, `:907`, `:952`, `:2085` ; fr-CH `:1856` (`invoice-default-revenue-account-unusable`)
    et son repli `InvoiceForm.svelte:768` ; `.env.example:305`. **Assumées : 14** — `README.md:213`
    (feuille de route publiée) ; cinq noms d'entité et d'action du journal d'audit (fr-CH `:2244`, `:2245`,
    `:2307`, `:2308`, `:2309` : `audit-log-entity-company-*-settings`, `audit-log-action-company-*-settings-*`)
    — des noms d'objet, non des renvois au menu ; **huit commentaires de code**, non affichés
    (`dunning/+page.svelte:2`, `dunning.api.ts:57`, `dunning.types.ts:34`, `write-off.ts:75`,
    `company_dunning_settings.rs:1`, `:11`, `supplier_invoices.rs:404`, `kesh-seed/src/lib.rs:370`).
    7 + 1 + 5 + 8 = 21. Exclus par `E` : `frontend/tests/e2e/dunning.spec.ts:6,26` (titres de test),
    `CHANGELOG.md` (notes publiées).
  - (B) `git grep -In 'réglages' -- . "${E[@]}"` → **132** lignes sur `245b91ee` : **38** dans les fichiers
    de texte (`README.md`, `docs/`, `website/`, `.env.example`, `DOCKER_START.md`, catalogues — commande :
    `git grep -In 'réglages' -- README.md docs website .env.example DOCKER_START.md crates/kesh-i18n/locales "${E[@]}"`),
    **94** dans le code.
    - Texte, **corrigées : 2** — `README.md:34` et fr-CH `:1887` (`error-invoice-pdf-header-overflow`),
      renvois à l'écran où l'on agit. **Assumées : 36**, « réglages » nom commun — les valeurs configurées,
      non le menu (C-15-14-11 ; C-15-14-17) : fr-CH `:31`, `:32`, `:33`
      (`error-settlement-{rounding,write-off,vat-payable}-account-is-receivable`, « désigné dans les réglages
      comme compte de … », qui renvoient ensuite à « Paramètres »), `:1718`, `:1752` (`dunning-load-error`,
      `dunning-settings-conflict`, sur l'écran même) ; quatre commentaires de catalogue (fr-CH `:1877`,
      de-CH `:1760`, en-CH `:1765`, it-CH `:1758`, non affichés) ; `docs/api-external.md:338`, `:359`,
      `:363`, `:365`, `:393`, `:508` (« désigné dans les réglages », « compte … des réglages ») ;
      `docs/i18n-glossaire.md:197` (l'entrée « réglages de facturation », qui atteste le nom commun) ;
      manuel d'administration `:1244`, `:1246`, `:1263`, `:2271`, `:2277` ; manuel utilisateur **15** —
      `:364`, `:380`, `:394` (« désigné dans les réglages », « les réglages de facturation refusent »),
      `:946`, `:965`, `:1155`, `:1266`, `:1303`, `:1307`, `:1538`, `:1910`, `:2048`, `:2235`, et les deux
      lignes de la 15-7b1, **`:185` et `:2248` sur `245b91ee`** (liste du mode exploration, entrée de
      journal du chargement de démonstration). 5 + 4 + 6 + 1 + 5 + 15 = 36 ; 2 + 36 = 38.
      **`:965`** (« La valeur complète reste intacte dans vos réglages ») est tranché **assumé** (validation
      P2, F-6) : il dit où la valeur est **conservée**, il ne prescrit aucun geste.
    - Code, **corrigée : 1** — le repli `errors.rs:1807` d'`error-invoice-pdf-header-overflow`.
      **Assumées : 93** — cinq replis égaux à des clés assumées (`errors.rs:3713`, `:3719`, `:3725` ;
      `settings/dunning/+page.svelte:73`, `:170`) ; deux messages non affichés (`kesh-db/src/errors.rs:644`,
      le `Display` de `DbError::DesignatedAccountsNotPostable`, que `kesh-api/src/errors.rs:3290` traduit en
      code ; `company_invoice_settings.rs:313`, un `tracing::warn!`) ; **86 commentaires** — 84 au détecteur
      de la convention, plus deux continuations de commentaire HTML (`InvoiceForm.svelte:736`, `:821`).
      1 + 5 + 2 + 86 = 94. Contrôle : `git grep -In 'réglages' -- . "${E[@]}" ':(exclude)README.md' ':(exclude)docs' ':(exclude)website' ':(exclude).env.example' ':(exclude)DOCKER_START.md' ':(exclude)crates/kesh-i18n/locales' | grep -cE '^[^:]+:[0-9]+:\s*(//|\*|/\*|<!--|--|#)'` → 84.
  - L'ancienne commande (B), bornée à quatre fichiers, rend 30 sur `245b91ee` (28 sur `bcded0c8`, plus les
    deux lignes de la 15-7b1) : ce sont les 30 lignes de manuels, fr-CH et `errors.rs` ci-dessus.
- **Tests** : G2 (`aucun_renvoi_au_menu_reglages`, garde documentaire : aucun `\emph{Réglages}` ni
  `Réglages et` dans `docs/manual/fr/*.tex`, aucun `(Réglages)` dans `.env.example`, aucun `dans les
  réglages` dans `README.md` — positif : `README.md` contient `dans les Paramètres`) ; G3
  (`le_message_du_compte_de_produit_renvoie_a_l_ecran_reel`, kesh-i18n : pour chaque locale, la valeur de
  `invoice-default-revenue-account-unusable` contient `settings-invoicing-title` de la même locale, ` — `
  remplacé par ` → ` ; et la valeur fr-CH de `error-invoice-pdf-header-overflow` contient `dans les
  Paramètres` et non `dans les réglages`) ; le repli Rust de `error-invoice-pdf-header-overflow` est gardé
  par G9, le repli de `InvoiceForm.svelte` par G13. Mutations : remettre « dans les Réglages » au
  catalogue fr → G3 rouge ; remettre « dans les réglages » à `error-invoice-pdf-header-overflow` (fr-CH)
  → G3 rouge ; remettre `\emph{Réglages}` au manuel → G2 rouge ; remettre `(Réglages)` à `.env.example`
  → G2 rouge ; remettre « dans les réglages » à `README.md:34` → G2 rouge ; remettre « dans les
  Réglages » au repli d'`InvoiceForm.svelte` → G13 rouge.

### AC 3 — Les plans comptables décrits sont les trois que Kesh livre (#488, C-15-14-3)

Kesh livre **trois** plans, choisis par le **type d'organisation** à l'onboarding — PME, indépendant,
association (`crates/kesh-core/assets/charts/{pme,independant,association}.json`, `load_chart(org_type)`,
`routes/onboarding.rs:444`). Aucun fichier ne porte « Sterchi » ni « KMU » ; **aucun import de plan**
n'existe (aucune route, aucun écran).

- `user-manual.tex:308` : « Kesh fournit trois plans comptables, dont la structure suit le plan comptable
  PME suisse : un pour les PME, un pour les indépendants, un pour les associations. Le plan est choisi
  par le type d'organisation que vous indiquez à l'onboarding ; vous l'adaptez ensuite en ajoutant,
  modifiant ou archivant des comptes. »
- `:204` : « Kesh met en place le plan comptable de votre type d'organisation et votre premier exercice. »
- `:314` (légende de capture) : « Plan comptable PME affiché par classes hiérarchiques. »
- `:426`, `:437` : « Le plan PME », « les comptes du plan que vous n'utilisez pas ».
- **Sous-section « Import d'un plan personnalisé » (`:409-422`) retirée** — fonction fictive, même
  symptôme que #291 (C-15-14-3) ; si un renvoi `\ref` y pointe, le retirer aussi (`grep -n 'import.*plan\|sec:import-plan'`).
- `admin-manual.tex:79` : « avec les plans comptables suisses PME, indépendant et association » ;
  `admin-manual.tex:1375-1386` (`bcded0c8` ; `:1361-1372` sur `dc4bc58b`) (tableau « Choix du plan comptable ») réécrit sur les trois plans réels et
  leur choix par le type d'organisation, **sans** la ligne « Personnalisé CSV » ; le `keshtip` qui suit
  reste juste (le plan se modifie après création) — à relire.
- `marketing-brochure.tex:260` : « Plan comptable suisse PME, adapté à votre forme juridique. » ;
  `:385` : « Plan comptable suisse (PME, indépendant, association) avec saisie d'écritures en partie double. »
- `user-manual.tex:182` (« Une company fictive avec un plan comptable Sterchi PME ») : **corrigé par la
  15-7b1**, **mergée** (`245b91ee`, PR #583 — validation P3 : la dépendance de C-15-14-19 est satisfaite).
  Constaté sur `245b91ee` : la ligne `:183` porte « \item Le plan comptable PME, dans la langue de
  l'installation. », et `grep -c 'Sterchi' docs/manual/fr/user-manual.tex` → 5 (`:205`, `:309`, `:315`,
  `:427`, `:438` sur `245b91ee`), tous corrigés ci-dessous. Ne rien faire aux lignes `:173`-`:190`.
- **`docs/user-guide/fr/getting-started.md:29`** (validation P2, F-4 — guide lié depuis `README.md:124`) :
  « 4. **Plan comptable** (PME Suisse / Indépendant / KMU / Verein, ou import CSV custom *(à venir
  v0.2)*). » décrit une étape inexistante, deux plans fictifs et un import de plan. **Retirée** ; l'étape 1
  (`:26`) devient « **Choix du type d'organisation** (indépendant, PME, association) — il détermine le plan
  comptable que Kesh met en place. » et les étapes suivantes se renumérotent. Les autres étapes du guide ne
  sont **pas** auditées contre l'onboarding réel (hors #488) : angle mort déclaré.
- **Capture `plan-comptable.png`** (`:314`) : le fichier n'existe pas (`docs/manual/fr/screenshots/` ne
  contient que `_placeholder.png`) ; seule la légende porte le texte, corrigée ci-dessus (F-14 réfuté).
- **Liens externes conservés** : `admin-manual.tex:2516` (`:2405` sur `dc4bc58b` — validation P2, R-9 =
  L-2), `user-manual.tex:2461` (URL `kmu.admin.ch`, en minuscules : la commande ci-dessous, sensible à la
  casse, ne les rend pas).
- **Inventaire** : `git grep -InE 'Sterchi|KMU|pré-configur|Import CSV' -- . "${E[@]}"`
  → **16** lignes sur `245b91ee` (validation P3 : 18 sur `bcded0c8` avec l'ancienne commande bornée,
  dont `:182`, corrigé depuis par la 15-7b1, et `docs/change_request.md:40`, `:43`, désormais exclus par
  `E`). **Corrigées : 14** — brochure `:260`, `:385` ; manuel d'administration `:79`, `:1375`, `:1381`,
  `:1382` (tableau, dont la ligne `:1383` « Personnalisé CSV », hors motif, part avec lui) ; manuel
  utilisateur `:204`, `:308`, `:314`, `:411` (sous-section retirée), `:426`, `:437`, `:859` (AC 4) ;
  guide `:29`. **Assumées : 2** — `user-manual.tex:1588` (« Import CSV multi-encodage » : l'import
  **bancaire**, réel) ; de-CH `:106` (`onboarding-org-pme = KMU` : « KMU » est le mot allemand pour PME,
  libellé du type d'organisation, non un plan). 14 + 2 = 16.
- **Test** G4 (`plans_comptables_reels`) : aucun `Sterchi`, aucun `KMU` hors URL dans `docs/manual/fr/*.tex`,
  `README.md` **et `docs/user-guide/fr/getting-started.md`** ; aucun `\emph{Import CSV}` (forme des deux
  imports fictifs — l'import bancaire s'écrit `\emph{Nouvel import CSV}` et reste permis) ni `import CSV
  custom` (guide). Mutations : remettre « Sterchi PME » à `:308` → rouge ; remettre « KMU » à la ligne du
  guide → rouge.

### AC 4 — Le manuel ne promet plus d'import de contacts (#291, C-15-14-3)

- `user-manual.tex:857-859` (`\subsubsection{Import en masse}` et sa phrase) **retirés**. Vérifié sur
  `dc4bc58b` : aucune route `/api/v1/contacts/import`, aucun bouton d'import à
  `frontend/src/routes/(app)/contacts/+page.svelte`.
- Couvert par G4 (`\emph{Import CSV}` interdit). Mutation : remettre la sous-section → rouge.

### AC 5 — L'import de factures se dit déclenché à la main (#458, refs #459)

Aucun processus ne surveille `KESH_INBOX_DIR` (vérifié : aucune tâche de fond dans `kesh-api` ;
`inbox_import.rs` n'est appelé que par `POST /api/v1/inbox-import`).

- `user-manual.tex:1524` : « … déposées dans le dossier d'import (voir le manuel administrateur pour sa
  configuration). L'import ne se déclenche pas tout seul : vous le lancez depuis l'écran décrit
  ci-dessous. » ⚠️ Cette phrase deviendra fausse **sans signal** le jour où #459 automatisera l'import
  (G5 rougira, lui, mais pas sur cette phrase) : l'orchestrateur a commenté #459 (validation P4, L-5 de
  la lentille R) pour nommer G5 et la phrase à inverser. (le paragraphe suivant, `:1527-1528`, nomme déjà \emph{Quotidien} → \emph{Importer des
  factures} — validation P1, R7 : pas de redite.)
- `marketing-brochure.tex:396` : « Import de factures fournisseurs déposées dans un dossier, avec décodage
  du QR-facture côté serveur. »
- `README.md:43` : « dépôt de factures … dans un dossier d'import, import lancé depuis l'écran, décodage … ».
- `.env.example:197` (`bcded0c8` ; `:194` sur `dc4bc58b`) : « … déposées dans un dossier surveillé, décoder
  le QR … » → « … déposées dans un dossier, importées à la demande depuis l'écran « Importer des
  factures », décoder le QR … ». Validation P1, F-4 : c'est le fichier que l'exploitant lit pour configurer l'inbox.
  `configuration_transmise.rs` lit `.env.example` (ses lignes d'affectation et les commentaires qui les
  précèdent) : relancer ce binaire après la modification.
- **Assumé** : `README.md:211` (feuille de route v0.4.0 publiée — on ne réécrit pas l'historique ; seule
  la forme de ses références d'issues change, AC 7) ; `CHANGELOG.md:499` (notes de la 0.4.0, publiées). Les
  « Surveiller les logs / CVE » du manuel d'administration sont d'un autre sens. « Dossier inbox **scruté à
  l'import** » (`admin-manual.tex:781`, `.env.example:204`) : dit que le dossier est lu **au moment de**
  l'import, ce qui est exact — conservé (validation P1, R7 ; C-15-14-12).
- **Inventaire** : `git grep -IniE 'surveill|scrut' -- . "${E[@]}"` → **25** lignes sur `245b91ee`
  (validation P3 : la commande bornée de la P2 rendait 13, dont `CHANGELOG.md:499`, désormais exclu par
  `E`). **Corrigées : 4** — `README.md:43`, `.env.example:197`, `marketing-brochure.tex:396`,
  `user-manual.tex:1524`. **Assumées : 21** — `README.md:211` (feuille de route publiée) ; « scruté à
  l'import » `.env.example:204` et `admin-manual.tex:781` (exact, C-15-14-12) ; `admin-manual.tex:506`,
  `:1972`, `:2137`, `:2138`, `:2350` (« surveiller » des logs, des CVE, du renouvellement de certificats :
  autre sens) ; **13 commentaires de code ou de script**, autre sens (gardes de mise en page, mémoire,
  inbox « scruté à l'import ») : `kesh-api/src/config.rs:264`, `:891`, `:960`,
  `routes/invoice_pdf_service.rs:956`, `kesh-qrbill/src/pdf.rs:252`, `:432`, `:486`, `:514`, `:1133`,
  `:1435`, `kesh-qrbill/src/types.rs:398`, `kesh-report/src/trial_balance.rs:49`, `scripts/mem-guard.sh:6`.
  4 + 1 + 2 + 5 + 13 = 25.
- **Test** G5 (`aucun_dossier_surveille`) : la chaîne `dossier surveillé` n'apparaît dans aucun
  `docs/manual/fr/*.tex`, ni dans `.env.example`, ni dans `README.md` hors de la ligne de feuille de route
  `| v0.4.0 |`. Mutations : remettre « dossier surveillé » à `user-manual.tex` → rouge ; à `.env.example`
  → rouge.

### AC 6 — La correction de la KF-036 se dit livrée, dans la version qui l'a livrée (#449)

Le correctif (#167) est sous `## [0.10.0] — 2026-08-19` du CHANGELOG (`CHANGELOG.md:329`, section
`:303-332` sur `bcded0c8` ; `:321`, `:295-324` sur `dc4bc58b`).

- `docs/api-external.md:484` devient : « ⚠️ **Dans quelle version ?** Cette fermeture est livrée depuis la
  **v0.10.0** (entrée [#167](https://github.com/guycorbaz/kesh/issues/167) du [CHANGELOG](../CHANGELOG.md)).
  Jusqu'à la v0.9.0 incluse, une clé `read-write` créée par un Administrateur atteint les routes
  d'administration : sur une telle version, traitez une clé d'origine Administrateur comme un secret
  d'administrateur, et mettez Kesh à jour. »
- `admin-manual.tex:2035` (`bcded0c8` ; `:1931` sur `dc4bc58b`) : la phrase « \textbf{Attention à la version} : cette fermeture n'est pas dans la
  v0.9.0 --- voir l'encadré … » devient « Cette fermeture est livrée depuis la v0.10.0 --- voir l'encadré
  \emph{« Dans quelle version ? »} plus bas. »
- `admin-manual.tex:2051` (`bcded0c8` ; `:1947` sur `dc4bc58b`) (encadré) : titre « Dans quelle version ? Depuis la v0.10.0. » ; corps : la
  faille existait jusqu'à la v0.9.0 incluse ; le correctif est livré par la v0.10.0 (entrée \#167 du
  \keshcommand{CHANGELOG.md}) ; sur une version antérieure, la consigne de prudence. **Plus aucun renvoi à
  une section `[Unreleased]`** (elle se renomme à chaque release — c'est ce qui a périmé le texte).
- **Inventaire** : `git grep -InE 'Unreleased|v0\.9\.0' -- . "${E[@]}"` → **5** lignes sur `245b91ee`.
  **Corrigées : 3** (`api-external.md:484`, `admin-manual.tex:2035`, `:2051`). **Assumées : 2** — les
  lignes de feuille de route qui citent `v0.9.0` (`README.md:216`, `website/roadmap.html:277`), publiées.
  3 + 2 = 5. Après correction : **2**, et `git grep -IF 'Unreleased' -- . "${E[@]}"` → 0 (2 avant, les deux sites corrigés).
- **Test** G6 (`la_faille_kf036_n_est_pas_annoncee_ouverte`) : ni `Unreleased`, ni `n'est **pas** dans la
  v0.9.0` (forme Markdown), ni `n'est pas dans la v0.9.0` (forme LaTeX de `admin-manual.tex:2035` —
  validation P1, R11 = F-7), ni `Pas dans celle que décrit ce manuel` dans `docs/api-external.md` et
  `docs/manual/fr/admin-manual.tex` ; `v0.10.0` figure dans le paragraphe qui suit « Dans quelle version ».

### AC 7 — Toute référence d'issue du README est un lien (#432, C-15-14-6)

- Les 54 références `#NNN` du README (lignes 211-223 sur `dc4bc58b` : 32 `[#NNN]`, 22 nues) s'écrivent
  `[#NNN](https://github.com/guycorbaz/kesh/issues/NNN)`. Contenu des lignes inchangé par ailleurs.
- Périmètre : le **README seul** — l'objet de #432 ; les autres fichiers à références d'issues
  (`CHANGELOG.md`, fiches) sont hors de l'issue, et `website/` n'en porte aucune (`grep -noE '#[0-9]{2,4}\b' website/*.html` → le seul exemple `refs #42` d'`issues.html:128`).
- Recompte à la source avant et après : `grep -oE '#[0-9]{2,4}\b' README.md | wc -l` (54 avant) ;
  `grep -oE '\[#[0-9]+\]\(https://github.com/guycorbaz/kesh/issues/[0-9]+\)' README.md | wc -l` (54 après).
- **Test** G7 (`references_d_issues_du_readme_sont_des_liens`) : toute occurrence de `#\d+` du README
  est de la forme `[#N](https://github.com/guycorbaz/kesh/issues/N)` avec le **même** `N` des deux côtés ;
  le test compte aussi le nombre de liens (> 0, anti-test-muet). Mutation : remettre un `[#164]` nu → rouge ;
  écrire `[#164](…/issues/165)` → rouge.

### AC 8 — Les prescriptions de réouverture disent l'ordre (#569, C-15-14-4)

Forme (C111, complétée) : rouvrir les exercices clôturés **jusqu'à celui-ci, en commençant par le plus
récent**. Chaque clé garde son style d'apostrophe actuel ; tout repli est **octet pour octet** la valeur
fr-CH.

| Clé (relevé fr-CH `dc4bc58b`) | Texte fr-CH attendu |
|---|---|
| `settlement-cancel-blocked-fiscal-year-closed` (`:821`) | Ce règlement appartient à un exercice clôturé : pour pouvoir l'annuler, un administrateur doit rouvrir les exercices clôturés jusqu'à celui-ci, en commençant par le plus récent. |
| `reconciliation-cancel-blocked-fiscal-year-closed` (`:829`) | Ce rapprochement appartient à un exercice clôturé : pour pouvoir l'annuler, un administrateur doit rouvrir les exercices clôturés jusqu'à celui-ci, en commençant par le plus récent. |
| `supplier-invoices-cancel-blocked-fiscal-year-closed` (`:1966`) | Cette facture appartient à un exercice clôturé : pour pouvoir l'annuler, un administrateur doit rouvrir les exercices clôturés jusqu'à celui-ci, en commençant par le plus récent. |
| `error-fiscal-year-reopen-blocked` (`:933`) | Réouverture impossible : un exercice postérieur est clôturé ; rouvrez d’abord les exercices clôturés, en commençant par le plus récent. |
| `opening-balances-locked-first-year-closed` (`:964`) | Le premier exercice « { $name } » est clôturé : avant la saisie des soldes de départ, un administrateur doit rouvrir les exercices clôturés jusqu’à celui-ci, en commençant par le plus récent. |
| `error-opening-balances-first-year-closed` (`:972`) | Le premier exercice est clôturé : avant de saisir les soldes de départ, rouvrez les exercices clôturés jusqu’à celui-ci, en commençant par le plus récent. |

Trois autres locales, même structure (marqueurs d'ordre déjà employés par
`journal-entries-modify-blocked-later-fiscal-year-closed`) :

- de-CH : « … muss ein Administrator die abgeschlossenen Geschäftsjahre bis zu diesem wieder öffnen,
  beginnend mit dem neuesten » ; reopen-blocked : « Wiedereröffnung nicht möglich: Ein späteres
  Geschäftsjahr ist abgeschlossen; öffnen Sie zuerst die abgeschlossenen Geschäftsjahre wieder,
  beginnend mit dem neuesten. » (« wieder öffnen », le verbe du bouton et des clés voisines —
  `fiscal-year-reopen-button = Wieder öffnen`, `de-CH:871`, `:875`, `:773` — non « wieder eröffnen » ;
  validation P1, F-2.) **Deux verbes coexisteront dans la famille** (validation P2, L-3 ; C-15-14-20) :
  les quatre clés qui portent déjà le marqueur (`de-CH:389`, `:390`, `:853`, `:884`) écrivent « eröffnet
  eine Administratorin oder ein Administrator … wieder » ; les six de l'AC garderont « muss ein
  Administrator … wieder öffnen », chaque clé gardant sa forme (« Ein Administrator » est celle des six
  aujourd'hui). Assumé : le marqueur d'ordre, seul contrôlé par G8, est identique ; harmoniser les quatre
  autres clés sortirait de #569.
- it-CH : « … un amministratore deve riaprire gli esercizi chiusi fino a questo, cominciando dal più
  recente » ; reopen-blocked : « Riapertura impossibile: un esercizio successivo è chiuso; riapri prima
  gli esercizi chiusi, cominciando dal più recente. »
- en-CH : « … an administrator must reopen the closed fiscal years down to this one, starting with the
  most recent » ; reopen-blocked : « Cannot reopen: a later fiscal year is closed; first reopen the closed
  fiscal years, starting with the most recent. »

Le développeur écrit les 18 valeurs cibles complètes en reprenant le début actuel de chaque valeur (sujet,
« Diese Zahlung gehört … », etc.) ; seule la prescription change.

**Replis** — chacun égal à la nouvelle valeur fr-CH :

- Rust : `crates/kesh-api/src/errors.rs:3085`, `:3597`, `:3637` (et **retirer** les trois commentaires
  `// #569 : ce texte prescrit …` qui les précèdent, `:3080`, `:3592`, `:3632`) ;
  `crates/kesh-api/src/routes/fiscal_years.rs:188` ; `crates/kesh-api/src/routes/opening_balances.rs:205`, `:406`.
- Frontend : `frontend/src/lib/shared/utils/settlement-cancel-blocked.ts:39` ;
  `frontend/src/lib/features/reconciliation/reconciliation-cancel.ts:60` ;
  `frontend/src/lib/features/supplier-invoices/invoice-cancel.ts:43` ;
  `frontend/src/routes/(app)/settings/opening-balances/+page.svelte:382`.

**Tests qui figent l'ancien texte**, à basculer sur le neuf (le marqueur « en commençant par le plus
récent ») — inventaire `grep -rnE "rouvrir l" frontend/src --include=*.test.ts` → **8** lignes sur
`bcded0c8` : **7 sites dans 5 fichiers** — six assertions ou tables d'assertions
(`settlement-cancel-blocked.test.ts:34`, `invoice-settlements-page.test.ts:182` — `toContain("rouvrir l")`,
reste vert sur le texte neuf mais ne prouve plus l'ordre, validation P1, F-12 —,
`CancelReconciliationDialog.test.ts:91`, `reconciliation-cancel.test.ts:25` et `:61`,
`InvoiceSettlements.test.ts:79`) et **une donnée de mock** (`invoice-settlements-page.test.ts:173`, le
message serveur simulé, aligné sur la valeur neuve) — plus **1 assumée** :
`fiscal-years-page.test.ts:317`, un commentaire (« Jamais « rouvrir l'exercice clos le plus proche » »).
7 + 1 = 8 (validation P2, R-7 : la P1 écrivait « six assertions »).

**Documentation** :

- `user-manual.tex:1208-1209` (liste « l'annulation d'un règlement est refusée », même cas que `:1456` —
  validation P1, R1 = F-1) : « \item \textbf{l'exercice du règlement est clôturé}~: un administrateur doit
  d'abord \textbf{rouvrir} les exercices clôturés jusqu'à celui du règlement, en commençant par le plus
  récent~; ».
- `user-manual.tex:1456`, `:1478` : « (un administrateur doit d'abord rouvrir les exercices clôturés
  jusqu'à celui-ci, en commençant par le plus récent) » ; `:1773` : « un administrateur doit d'abord
  rouvrir les exercices clôturés jusqu'à celui-ci, en commençant par le plus récent. » ; `:2316-2317` :
  « un administrateur doit d'abord rouvrir les exercices clôturés jusqu'à celui de la pièce, en commençant
  par le plus récent. » `:2311` (« aucune raison de rouvrir l'exercice ») **décrit**, il reste. `:717`,
  `:722` (procédure de réouverture, qui dit déjà la garde d'ordre plus bas) restent.
- `docs/api-external.md:330` : « Règlement d'un exercice **clos** — un administrateur doit rouvrir les
  exercices clôturés jusqu'à celui-ci, en commençant par le plus récent ».
- **Inventaire, sur tout le dépôt suivi** (validation P2, R-6 = F-9 ; validation P3, C-15-14-25) :
  `git grep -InE '[Rr]ouvr|[Rr]éouv' -- . "${E[@]}"` → **153** lignes sur `245b91ee`, en trois parts :
  catalogue fr-CH **21**, documentation **26**, code **106** (commande du code :
  `git grep -InE '[Rr]ouvr|[Rr]éouv' -- crates frontend scripts ':(exclude)crates/kesh-i18n/locales' "${E[@]}"`).
  Les catalogues de-CH/it-CH/en-CH n'ont aucune forme française. 21 + 26 + 106 = 153.
  - **Catalogue fr-CH, 21** : les **20 clés** du domaine de G8 (partition 6 + 4 + 10 ci-dessous) et un
    commentaire (`:921`, en-tête de section, non affiché).
  - **Documentation, 26** — manuel utilisateur 16, manuel d'administration 5, `api-external.md` 2,
    README 2, `docs/optimistic-locking-patterns.md` 1. **Corrigées : 6** — utilisateur `:1209`, `:1456`,
    `:1478`, `:1773`, `:2317` ; `api-external.md:330`. **Disent déjà l'ordre : 5** — utilisateur `:698`,
    `:725`, `:740` ; administration `:1399` (« garde d'ordre »), `:1700`. **Décrivent ou nomment sans
    prescrire : 15** — utilisateur `:406` (rouvrir un **compte**), `:587`, `:600`, `:717` (renvoie à la
    procédure), `:720` (titre), `:722` (procédure, l'ordre suit à `:725`), `:1237` (« rouvrez l'aperçu »),
    `:2311` ; administration `:1396`, `:1398` (titre), `:2035` (liste des routes d'administration) ;
    `api-external.md:77` (même liste) ; README `:46`, `:215` (fonctionnalités, feuille de route) ;
    `optimistic-locking-patterns.md:7` (« deux utilisateurs qui rouvrent le même formulaire » : autre
    objet). 6 + 5 + 15 = 26.
  - **Code, 106** — **72 commentaires** au détecteur de la convention ; **34** autres lignes :
    - les **replis des six clés de l'AC**, corrigés : 10 — Rust `errors.rs:3085`, `:3597`, `:3637`,
      `routes/fiscal_years.rs:188`, `routes/opening_balances.rs:205`, `:406` ; frontend les quatre de la
      liste ci-dessus ;
    - les replis des **4 clés qui portent déjà le marqueur**, qui le portent aussi : 4 — `errors.rs:1539`,
      `:2925`, `journal-entries/blocker-messages.ts:94`, `settings/fiscal-years/+page.svelte:355` ;
    - les replis des **clés exemptées** : 12 — `errors.rs:1631` (`error-reminder-amounts-changed`),
      `routes/fiscal_years.rs:359`, `:365` (motif), `settings/fiscal-years/+page.svelte:78`, `:88`, `:435`,
      `:447`, `:585`, `:620`, `:626`, `:641`, `:668` ;
    - autre objet ou non affiché : 8 — `kesh-core/src/chart_of_accounts/mod.rs:1020`, `:1032`, `:1037`
      (un test sur un **compte**) ; `kesh-db/src/post_restore.rs:422` (justification d'exemption :
      `postable` « réouvrable ») ; `frontend/scripts/mutants-22-2b.mjs:145` (outil de mutation) ; trois
      continuations de commentaire HTML (`invoices/[id]/+page.svelte:1572`,
      `settings/fiscal-years/+page.svelte:346`, `:347`).
    10 + 4 + 12 + 8 = 34 ; 72 + 34 = 106. **Aucun repli hors de la liste ci-dessus ne prescrit la
    réouverture sans l'ordre.**

**Domaine et inventaire des sites non résolus** (C-15-14-14). Le verbe se cherche **en fr-CH seulement**,
où il est univoque : `grep -nE '[Rr]ouvr|[Rr]éouv' crates/kesh-i18n/locales/fr-CH/messages.ftl` (hors
commentaires) — **20 clés** sur `bcded0c8` et sur `245b91ee`. Le domaine du test est **l'ensemble de ces clés**, contrôlées
**dans les quatre locales** : en allemand, le verbe prend au moins cinq formes (« wieder öffnen »,
« öffnen Sie dieses zuerst », « Öffnen Sie es wieder », « Wiedereröffnung », « wieder eröffnet ») et un
motif par locale passerait à vide sur l'une d'elles (validation P1, R5 = F-2). Partition des 20 :

- **6 clés de l'AC** (ci-dessus) : doivent porter le marqueur dans les quatre locales ;
- **4 clés qui le portent déjà** : `journal-entries-modify-blocked-later-fiscal-year-closed`,
  `error-later-fiscal-year-closed`, `fiscal-year-out-of-order-warning`, `error-fiscal-year-create-later-closed`
  — contrôlées comme les six (la clause de contre-passation des deux premières est examinée et conservée :
  C-15-14-4, renvoi B-1 = E-1 de la 15-12b) ;
- **10 clés exemptées**, liste fermée, exemption **par clé** (donc dans les quatre locales) :
  - nomment l'acte sans le prescrire : `fiscal-year-reopen-button`, `fiscal-year-reopen-confirmation-title`,
    `fiscal-year-reopen-confirmation-action`, `fiscal-year-reopen-motif-label`,
    `error-fiscal-year-reopen-motif-empty`, `error-fiscal-year-reopen-motif-too-long` ;
  - décrivent, ne prescrivent pas : `fiscal-year-close-confirmation-body`, `fiscal-year-reopen-confirmation-body` ;
  - nomme l'exercice à rouvrir (le plus récent) : `fiscal-year-reopen-blocked-later-closed` ;
  - autre objet (« rouvrez l'aperçu ») : `error-reminder-amounts-changed` — exemptée dans les **quatre**
    locales (« Öffnen Sie die Vorschau erneut », « riaprire l'anteprima », « reopen the preview »).
- Hors domaine, parce qu'aucune forme fr ne matche (`rouvert` ne contient pas `rouvr`) :
  `fiscal-year-reopened`, `audit-log-action-fiscal-year-reopened` — participes qui décrivent l'acte fait.
- Recompte : 6 + 4 + 10 = 20. Tout autre site qu'un recompte révèle s'ajoute à la partition avec sa raison,
  ou se corrige.

**Test** G8 (`les_prescriptions_de_reouverture_disent_l_ordre`, kesh-i18n, `loader.rs` `mod tests`) :
le domaine = les clés dont la valeur **fr-CH** matche `[Rr]ouvr|[Rr]éouv` ; pour chacune, **hors** la liste
fermée des 10 exemptions, la valeur de **chaque** locale contient le marqueur d'ordre de la locale
(`en commençant par le plus récent` / `beginnend mit dem neuesten` / `cominciando dal più recente` /
`starting with the most recent`) ; chaque clé exemptée doit **encore** appartenir au domaine (sa valeur
fr-CH matche encore — sinon l'exemption est morte → rouge) ; les six clés de l'AC sont nommées, doivent
appartenir au domaine et porter le marqueur dans les quatre locales (anti-test-muet). G9
(`les_replis_rust_suivent_le_catalogue`, `textes_coherents.rs`) : chaque fichier Rust contient la valeur
fr-CH exacte de ses clés, selon cette table fermée (validation P2, R-8 = L-4 — **cinq** des six clés de
l'AC ont un repli Rust ; `opening-balances-locked-first-year-closed` n'a que le repli Svelte, gardé par
**G13** — et non par `i18n-repli-divergent-actif.test.ts`, qui ne voit pas une clé à site unique,
validation P3, R-1) :

| Fichier | Clés (sites sur `bcded0c8`) |
|---|---|
| `crates/kesh-api/src/errors.rs` | `settlement-cancel-blocked-fiscal-year-closed` (`:3085`), `reconciliation-cancel-blocked-fiscal-year-closed` (`:3597`), `supplier-invoices-cancel-blocked-fiscal-year-closed` (`:3637`), `error-invoice-pdf-header-overflow` (`:1807`, AC 2) |
| `crates/kesh-api/src/routes/fiscal_years.rs` | `error-fiscal-year-reopen-blocked` (`:188`) |
| `crates/kesh-api/src/routes/opening_balances.rs` | `error-opening-balances-first-year-closed` (`:205` **et** `:406` : la valeur doit y figurer deux fois) |

Six clés, sept sites. La comparaison se fait **après normalisation des continuations** de chaîne Rust (`\` en fin de ligne suivi du saut de
ligne et des blancs de tête retirés : le repli de `error-invoice-pdf-header-overflow` s'écrit sur cinq
lignes, et une comparaison brute passerait au rouge sur un repli juste) ; et aucune des chaînes
`rouvrir l'exercice pour`, `rouvrez-le`, `dans les réglages :`. Mutations : remettre l'ancienne valeur
fr-CH de `error-fiscal-year-reopen-blocked` → G8 rouge ; remettre l'ancienne valeur **de-CH** de la même
clé (« öffnen Sie dieses zuerst », sans « wieder ») → G8 rouge ; remettre l'ancien repli d'`errors.rs:3085`
→ G9 rouge ; remettre l'ancien repli de `invoice-cancel.ts:43` → **G13** rouge ; remettre « dans les
réglages » au repli d'`errors.rs:1807` → G9 rouge.

**Test** G13 (`les_replis_frontend_a_site_unique_suivent_le_catalogue`, Vitest, nouveau `describe` de
`frontend/src/lib/shared/i18n-repli-divergent-actif.test.ts`, qui possède déjà le relevé `replisParCle()`
— validation P3, R-1 ; C-15-14-26). La garde existante ne compare pas un repli au catalogue : elle ne retient
que les clés à **au moins deux** replis distincts (`parTexte.size > 1`, ligne 141), et chacune des cinq clés
ci-dessous n'a **qu'un** site d'appel — un repli resté à l'ancien texte n'y rougit pas. G13, table fermée de
cinq clés : `settlement-cancel-blocked-fiscal-year-closed`, `reconciliation-cancel-blocked-fiscal-year-closed`,
`supplier-invoices-cancel-blocked-fiscal-year-closed`, `opening-balances-locked-first-year-closed`,
`invoice-default-revenue-account-unusable` (AC 2). Pour chacune : `[...(releve.get(cle)?.keys() ?? [])]`
**égale** `[valeur fr-CH de la clé]` — lue dans `fr-CH/messages.ftl` (valeurs sur une ligne), comparée
telle quelle (même patron que l'assertion des deux titres d'avoir du même fichier, ligne 169). Le tableau à
un élément est l'anti-test-muet : un relevé vide, une clé disparue du catalogue (`undefined`) ou un second
repli divergent rougissent tous. Mutations : remettre l'ancien repli d'`invoice-cancel.ts:43` → rouge ;
remettre « dans les Réglages » au repli d'`InvoiceForm.svelte:768` → rouge ; remettre l'ancien repli
d'`opening-balances/+page.svelte:382` → rouge.

**Test** G12 (`le_manuel_dit_l_ordre_de_reouverture`, `textes_coherents.rs` — validation P2, R-6 = F-9 ;
C-15-14-8 appliqué à l'AC 8) : lit `docs/manual/fr/user-manual.tex` et `docs/api-external.md`, **normalisés**
(commandes `\textbf{…}`, `\emph{…}`, `\texttt{…}`, `\textit{…}`, `\keshcommand{…}`, `\keshpath{…}`
dépliées **jusqu'à stabilité** — imbrications comprises, validation P3, F-4 —, `~` → espace, blancs et
sauts de ligne réduits à une espace — les phrases visées sont coupées sur deux lignes) ; aucune occurrence de `doit d'abord le
rouvrir`, `doit d'abord rouvrir l'exercice` (manuel), ni de `doit le rouvrir` (`api-external.md`) ;
**positif** : `en commençant par le plus récent` figure **au moins 7 fois** dans le manuel normalisé
(2 aujourd'hui, `:698` et `:740`, + 5 corrigés) et au moins une fois dans `api-external.md`.
**Présent avant**, vérifié en validation P2 sur `bcded0c8`, et en validation P3 sur `245b91ee` avec la normalisation bouclée : `doit d'abord le
rouvrir` → 3, `doit d'abord rouvrir l'exercice` → 2, `en commençant par le plus récent` → 2 ; `doit le
rouvrir` (`api-external.md`) → 1. Mutations : remettre l'ancien `:1209` → rouge ; remettre l'ancienne
ligne `api-external.md:330` → rouge.

### AC 9 — `MWST` partout en de-CH (#321)

- de-CH : `invoice-line-vat-rate = MWST %` ; `invoice-error-vat-invalid = MWST-Satz nicht erlaubt. …` ;
  `error-invoice-too-many-lines-for-pdf` : « mit der MWST-Zusammenfassung » ; `reports-vat = MWST` ;
  `reports-vat-column-vat-due = Geschuldete MWST` ; commentaire `# MWST-Bericht (Story 11-2)`.
  Relevé sur `dc4bc58b` : `de-CH/messages.ftl:565`, `:608`, `:680`, `:1293`, `:1294`, `:1297`.
- **Ne pas propager** aux autres locales (IVA, TVA, VAT sont justes — avertissement de l'issue).
- **Inventaire** : `git grep -InE '\bMwSt\b' -- . "${E[@]}"` (replis compris) → **6** lignes sur
  `245b91ee` (6 aussi sur `bcded0c8`), toutes en de-CH, 6 corrigées, 0 assumée.
- **Test** G10 (`glossaire_mwst`, kesh-i18n) : aucune valeur de-CH ne matche `\bMwSt\b` ; au moins une
  valeur contient `MWST` (anti-test-muet). Mutation : remettre `reports-vat = MwSt` → rouge.

### AC 10 — Clôturer un exercice n'emprunte plus le verbe des panneaux (#323, C-15-14-7)

| Clé | de-CH | it-CH | en-CH |
|---|---|---|---|
| `fiscal-year-close-button` | Abschliessen | Chiudi l’esercizio | Close fiscal year |
| `fiscal-year-close-confirmation-action` | Abschliessen | Chiudi l’esercizio | Close fiscal year |
| `fiscal-year-close-confirmation-title` | Geschäftsjahr abschliessen? | (inchangé) | (inchangé) |
| `fiscal-year-close-confirmation-body` | « … das Geschäftsjahr „{ $name }“ abzuschliessen. Solange es abgeschlossen bleibt, … » | (inchangé) | (inchangé) |

- de-CH `settings-fiscal-years-link` (`:895`, « Erstellen, umbenennen oder schliessen Sie die
  Geschäftsjahre Ihres Unternehmens. ») : « Geschäftsjahre Ihres Unternehmens erstellen, umbenennen oder
  abschliessen. » (validation P1, F-13). it-CH et en-CH (« chiudi gli esercizi », « close your company
  fiscal years ») restent : la phrase nomme les exercices, le verbe n'y est pas ambigu.
- **Inventaire de-CH** (validation P2, R-1 = F-5 : la P1 affirmait ici « le seul autre site de-CH », ce
  qui était faux) : `grep -nE '\b([Gg]eschlossen|[Ss]chliessen|[Ss]chliessung)\b' crates/kesh-i18n/locales/de-CH/messages.ftl | grep -vE '^[0-9]+:#'`
  → **20** lignes sur `bcded0c8` et `245b91ee` (`\b` écarte « abgeschlossen », « abschliessen »,
  « einschliessen »). Sur tout le dépôt suivi (validation P3, C-15-14-25) :
  `git grep -InE '\b([Gg]eschlossen|[Ss]chliessen|[Ss]chliessung)\b' -- . "${E[@]}"` → **21** : ces 20
  lignes et `docs/i18n-glossaire.md:102`, la cellule que l'AC réécrit ci-dessous (elle nommera encore le
  verbe générique, réservé aux panneaux — elle reste donc au compte après correction). Aucun repli ni
  test ne porte ces formes (`frontend/src` n'a que des replis français). Partition des 20 :
  - **corrigées par le tableau ci-dessus et `settings-fiscal-years-link` : 5** — `:863`, `:864`, `:865`,
    `:866`, `:895` ;
  - **corrigée par l'AC 8 : 1** — `:882` (`error-fiscal-year-reopen-blocked`, « ist abgeschlossen ») ;
  - **corrigées ici, le participe suit le verbe : 6** (C-15-14-18) — la story réécrit déjà « geschlossen
    bleibt » en « abgeschlossen bleibt » (`:865`) et « ist geschlossen » en « ist abgeschlossen » (`:882`) ;
    le même critère vaut pour l'état qu'atteint un exercice clôturé, et le glossaire fixe « clôture =
    **Abschluss** » (`docs/i18n-glossaire.md:102`), que le catalogue emploie déjà (`:241`, `:288`,
    `:2224` `audit-log-action-fiscal-year-closed = Geschäftsjahr abgeschlossen`) :
    `fiscal-year-status-closed` (`:861`) `Geschlossen` → `Abgeschlossen` ; `fiscal-year-closed` (`:869`,
    le message qui suit le clic sur « Abschliessen ») → « Geschäftsjahr abgeschlossen. » ;
    `fiscal-year-reopen-confirmation-body` (`:873`) « bis zu einer erneuten Schliessung » → « bis zu
    einem erneuten Abschluss » ; `fiscal-year-reopen-blocked-later-closed` (`:877`) « noch geschlossen
    ist » → « noch abgeschlossen ist » ; `error-fiscal-year-already-closed` (`:890`) → « … ist bereits
    abgeschlossen. » ; `error-fiscal-year-closed-for-date` (`:893`) « ist geschlossen » → « ist
    abgeschlossen » — le reste de chaque valeur inchangé. Aucun test ni repli ne fige ces six valeurs
    (`grep -rnE "Geschlossen|geschlossen\.|erneuten Schliessung|bereits geschlossen" frontend/src frontend/tests crates --include=*.ts --include=*.svelte --include=*.rs`
    → 0 hors catalogues) ;
  - **assumées, verbe séparable « Schliessen Sie … ab » (= abschliessen) : 4** — `:162` (onboarding),
    `:853`, `:878`, `:883` ;
  - **assumées, fermeture d'un panneau — le sens que #323 réserve à « Schliessen » : 4** — `:790`
    (`reconciliation-cancel-dismiss`), `:1481`, `:1975`, `:2112` (`*-close`).
  5 + 1 + 6 + 4 + 4 = 20. Après correction, la commande de-CH rend **8** lignes (les deux groupes
  assumés), celle du dépôt **9** (avec le glossaire).
  it-CH (« chiuso ») et en-CH (« closed ») ne sont pas inventoriés : le participe y est univoque, seule
  l'action prêtait à confusion.
- **Glossaire** `docs/i18n-glossaire.md:102` (validation P1, R4) : la cellule « ⚠️⚠️ NE PAS confondre avec
  « fermer » … c'est KF-041 (#323), et `fiscal-year-close-button` porte encore la confusion » devient fausse
  par cette story. La réécrire : la confusion est **corrigée** (lien #323 conservé) ; les verbes retenus
  sont **Abschliessen / Chiudi l’esercizio / Close fiscal year**, le verbe générique (`Schliessen` /
  `Chiudi` / `Close`) étant réservé aux panneaux. Le nombre d'entrées de la partie A (76) ne change pas.
- fr-CH inchangé (« Clôturer ») : la suite E2E, en français, ne voit rien.
- Vérifier la collision avant d'écrire : `grep -nE '= (Abschliessen|Chiudi l’esercizio|Close fiscal year)$' crates/kesh-i18n/locales/*/messages.ftl`
  doit ne rendre que les clés ci-dessus.
- Vérifier que le bouton n'a pas de repli dans une autre langue que le français et qu'aucun test ne
  fige `Schliessen`/`Chiudi`/`Close` pour ces clés (`grep -rn 'fiscal-year-close-button' frontend/src crates docs`
  — `docs/` compris, pour le glossaire).
- **Test** G11 (`la_cloture_d_exercice_ne_parle_pas_comme_un_panneau`, kesh-i18n) : pour de-CH, it-CH,
  en-CH, les valeurs de `fiscal-year-close-button` et `fiscal-year-close-confirmation-action` diffèrent de
  celles de toutes les clés `*-close` et `*-dismiss` de la locale ; et elles égalent les valeurs du
  tableau. **De plus, en de-CH** (validation P2) : toute valeur qui matche `\b([Gg]eschlossen|[Ss]chliessen|[Ss]chliessung)\b`
  appartient à une clé `*-close`/`*-dismiss` **ou** porte la forme séparable `[Ss]chliessen Sie [^.;:]* ab\b`
  — et au moins une valeur de chaque groupe existe (anti-test-muet). Mutations : remettre `Schliessen` au
  bouton → rouge ; remettre `fiscal-year-closed = Geschäftsjahr geschlossen.` → rouge.

### AC 11 — CHANGELOG et feuille de route

- `CHANGELOG.md`, `## [0.13.0] — Non publié`, `### Corrigé` : une entrée « **Manuels et libellés : ce que
  Kesh fait réellement** » qui énumère, avec le lien de chaque issue — taux de TVA du manuel (#539) ;
  menu « Paramètres » (#547) ; trois plans comptables, sans import de plan ni de contacts fictifs (#488,
  #291) ; import de factures déclenché à la main (#458) ; KF-036 corrigée depuis la v0.10.0 (#449) ;
  liens d'issues du README (#432) ; messages de réouverture qui disent l'ordre (#569) ; `MWST` (#321) ;
  clôture d'exercice distincte de la fermeture d'un panneau, et dite « abgeschlossen » partout en
  allemand (#323).
- README « Feuille de route » : rien à changer (aucune fonctionnalité livrée ni retirée du tableau) ;
  vérifier seulement que la section « Fonctionnalités » ne promet plus de surveillance de dossier (AC 5).

## Tasks / Subtasks

> Les tâches s'appellent **T0-T8**, les tests **G1-G13** (« garde ») depuis la validation P2 (R-14 ; G13
> ajouté en validation P3) ; les Change Logs et le registre antérieurs écrivent les tests « T1-T11 » — même
> numéro, même test. La 15-14b numérote les siens **G14-G18** (validation P3, F-6), pour qu'un « G5 » ne
> désigne qu'un test.

- [x] **T0 — Rebase et relevé** (AC tous)
  - [x] `git fetch && git rebase origin/main` ; la 15-13a (`bcded0c8`) et la **15-7b1** (`245b91ee`, PR
        #583) sont mergées — la dépendance déclarée (C-15-14-19) est **satisfaite** depuis la validation P3 ;
        la branche de spécification est rebasée sur `245b91ee`. Si la 15-13b (PR #584) est mergée à son tour,
        re-trouver chaque site **par la valeur** (commandes des AC) et noter les écarts au Dev Agent Record.
        La 15-7b1 a touché `vat_rates.rs` (doc-comment `:351-353`), où T5 ajoute un `mod tests` : zones
        disjointes, mais c'est un repository — **gate complet** (exception `kesh-db`, validation P2, L-8).
  - [x] **PDF** (validation P1, F-8 ; R13) : la 15-7b1 modifie `admin-manual.pdf` et `user-manual.pdf`, la
        15-13b le manuel d'administration — un conflit sur un PDF ne se résout **jamais** à la main : prendre
        les `.tex` résolus, puis **régénérer** les trois PDF (`make -B fr`). Le `.tex` d'un manuel ne se fusionne
        pas non plus de confiance : relire chaque hunk en conflit contre les deux fiches.
  - [x] **Relancer chaque commande d'inventaire** des AC — sur tout le dépôt suivi moins `E`, `LC_ALL=C.UTF-8`
        — et comparer au compte écrit, relevé sur `245b91ee` (AC 1 : 4 ; AC 2 : 21 et 132 ; AC 3 : 16 ;
        AC 5 : 25 ; AC 6 : 5 ; AC 7 : 54 ; AC 8 : 20 clés fr-CH, 153 lignes sur le dépôt, 8 lignes Vitest ;
        AC 9 : 6 ; AC 10 : 20 en de-CH, 21 sur le dépôt) ; tout écart se ventile (corrigé ou assumé, avec sa
        raison) **avant** d'écrire.
  - [x] **Contrôle « présent avant »** : régénérer d'abord les PDF de l'état de départ (`make -B fr` — celui de
        `bcded0c8` était en retard sur son `.tex` ; `-B` parce que la règle `%.pdf: %.tex` ne rebâtit qu'un
        PDF plus ancien que son `.tex`, et qu'après un `checkout` ou un `rebase` les deux portent la date
        de l'écriture par git — constaté en P4 sur la brochure ; validation P4, L-2 de la lentille F), puis la fonction `occ` de la convention pour chaque
        ancien texte dont T1 contrôlera l'absence ; un motif qui rend 0 ici est à réécrire.
- [x] **T1 — Manuel utilisateur, brochure, manuel d'administration, guide** (AC 1-6, 8)
  - [x] AC 1 (2 sites), AC 2 (4 sites), AC 3 (user 204/308/314/409-422/426/437 — 182 fait par la 15-7b1 —,
        admin 79/1375-1386, brochure 260/385, guide `getting-started.md:26-32`), AC 4 (857-859), AC 5 (user
        1524, brochure 396), AC 6 (admin 2035/2051), AC 8 (user 1208-1209/1456/1478/1773/2316-2317) —
        numéros de `bcded0c8`.
  - [x] `scripts/mem-guard.sh make -B -C docs/manual fr` ; contrôle `occ` de chaque texte neuf (présent, ≥ 1)
        et ancien (absent, 0) dans les trois PDF.
- [x] **T2 — `api-external.md`, README (dont `:34`, AC 2), `.env.example`, glossaire** (AC 2, 5, 6, 7, 8, 10)
  - [x] **Contrôle par la valeur du glossaire** (validation P4, L-8 de la lentille R : aucune garde ne lit
        `docs/i18n-glossaire.md`, G11 lit les catalogues) : `grep -nF 'porte encore la confusion'
        docs/i18n-glossaire.md` → 0 ; `grep -nF '#323' docs/i18n-glossaire.md` → au moins la ligne réécrite ;
        sortie consignée au Dev Agent Record.
- [x] **T3 — Catalogues** (AC 2, 8, 9, 10 — dont les six participes de-CH de l'AC 10) — 4 locales
      ensemble, parité verte.
- [x] **T4 — Replis** (AC 2, 8) — Rust (7 sites, 6 clés : table de G9) et frontend (5) égaux au fr-CH,
      apostrophes du catalogue ; retirer les 3 commentaires `#569`.
- [x] **T5 — Tests** : G1 (`vat_rates.rs`, `mod tests` neuf — C-15-14-13), G3/G8/G10/G11
      (`kesh-i18n/src/loader.rs` `mod tests`), G2/G4/G5/G6/G7/G9/G12 (nouveau
      `crates/kesh-api/tests/textes_coherents.rs`, sans base) ; Vitest : G13 (nouveau `describe` de
      `i18n-repli-divergent-actif.test.ts`) et les 7 sites des 5 fichiers de l'AC 8 (six assertions et une
      donnée de mock).
  - [x] **Dépendance `regex`** (validation P3, L-1 ; C-15-14-27) : ni `kesh-api` ni `kesh-i18n` ne
        l'ont (`grep -c regex crates/kesh-api/Cargo.toml crates/kesh-i18n/Cargo.toml` → 0 et 0) ; l'ajouter
        en **`[dev-dependencies]`** des deux (`regex = "1.10"`, la version de `kesh-db` et `kesh-import`, déjà
        verrouillée dans `Cargo.lock`) — aucune dépendance de production. Le crate `regex` n'a **pas** de
        lookahead : une garde qui compte des occurrences qui se recouvrent (G18 de la 15-14b) boucle sur
        chaque alternative séparément, ou avance d'un caractère après chaque correspondance.
- [x] **T6 — Mutations** : chacune de la liste ci-dessous observée **rouge**, puis restaurée (`git checkout`
      **puis `touch`** du fichier — mémoire *Mutation restaurée, binaire périmé*).
- [x] **T7 — CHANGELOG** (AC 11).
- [x] **T8 — Gates** : fmt + clippy workspace ; `scripts/test-fast.sh` (gate complet : la story touche
      `kesh-api` et un repository de `kesh-db` — **exception `kesh-db` du `CLAUDE.md` : tout patch de revue
      qui touche `vat_rates.rs` impose le gate complet, même en cours de boucle** ; un patch qui ne touche
      que `textes_coherents.rs` ou les catalogues relève du gate ciblé, C-15-14-13) ; frontend `npm run check`, `lint-i18n-ownership`, `test:unit`, `build` ; **E2E complet au
      dernier commit de code** (D7) — les libellés fr changent sur des écrans E2E (annulations refusées,
      soldes de départ), juger chaque rouge contre `docs/testing.md` § « Les échecs attendus ».

## Tests et mutations

| Test | Lieu | Mutation qui doit le faire rougir |
|---|---|---|
| G1 `le_manuel_cite_les_taux_poses_par_kesh` | `kesh-db/src/repositories/vat_rates.rs` (`#[cfg(test)] mod tests` neuf) | `3.7\%` réécrit au manuel ; `380` → `370` dans `DEFAULT_SWISS_RATES` |
| G2 `aucun_renvoi_au_menu_reglages` | `kesh-api/tests/textes_coherents.rs` | `\emph{Réglages}` remis à `user-manual.tex` ; `(Réglages)` remis à `.env.example` ; « dans les réglages » remis à `README.md:34` |
| G3 `le_message_du_compte_de_produit_renvoie_a_l_ecran_reel` | `kesh-i18n/src/loader.rs` | « dans les Réglages » remis en fr-CH ; « dans les réglages » remis à `error-invoice-pdf-header-overflow` (fr-CH) |
| G4 `plans_comptables_reels` | `textes_coherents.rs` | « Sterchi PME » remis ; `\emph{Import CSV}` remis (contacts) ; « KMU » remis au guide |
| G5 `aucun_dossier_surveille` | `textes_coherents.rs` | « dossier surveillé » remis à `user-manual.tex` ; à `.env.example` |
| G6 `la_faille_kf036_n_est_pas_annoncee_ouverte` | `textes_coherents.rs` | `[Unreleased]` remis à `api-external.md` ; « n'est pas dans la v0.9.0 » remis à `admin-manual.tex` |
| G7 `references_d_issues_du_readme_sont_des_liens` | `textes_coherents.rs` | un `[#164]` nu ; un lien au mauvais numéro |
| G8 `les_prescriptions_de_reouverture_disent_l_ordre` | `loader.rs` | ancienne valeur fr-CH de `error-fiscal-year-reopen-blocked` ; ancienne valeur **de-CH** de la même clé ; exemption d'une clé dont la valeur fr-CH ne matche plus le verbe |
| G9 `les_replis_rust_suivent_le_catalogue` | `textes_coherents.rs` | ancien repli d'`errors.rs:3085` ; « dans les réglages » remis au repli d'`errors.rs:1807` |
| G10 `glossaire_mwst` | `loader.rs` | `reports-vat = MwSt` |
| G11 `la_cloture_d_exercice_ne_parle_pas_comme_un_panneau` | `loader.rs` | `fiscal-year-close-button = Schliessen` ; `fiscal-year-closed = Geschäftsjahr geschlossen.` |
| G12 `le_manuel_dit_l_ordre_de_reouverture` | `textes_coherents.rs` | ancien `user-manual.tex:1209` ; ancienne ligne `api-external.md:330` |
| G13 `les_replis_frontend_a_site_unique_suivent_le_catalogue` | `frontend/src/lib/shared/i18n-repli-divergent-actif.test.ts` (Vitest) | ancien repli d'`invoice-cancel.ts:43` ; « dans les Réglages » remis au repli d'`InvoiceForm.svelte:768` ; ancien repli d'`opening-balances/+page.svelte:382` |
| Vitest (5 fichiers de l'AC 8, 7 sites : 6 assertions, 1 donnée de mock) | frontend | ancien repli de `settlement-cancel-blocked.ts:39` |

`textes_coherents.rs` lit ses fichiers par `env!("CARGO_MANIFEST_DIR")` + `../../` ; chaque assertion
négative s'accompagne d'une assertion positive sur le même fichier (le fichier a été lu, il contient la
section attendue) : un chemin faux ne doit pas rendre un vert muet (mémoire *Tests qui prouvent moins*).

## Dev Notes

### Modules touchés, et dérogation à la règle de découpage

Paquets de code : **kesh-i18n** (catalogues, tests), **kesh-api** (`errors.rs`, `routes/fiscal_years.rs`,
`routes/opening_balances.rs`, un fichier de test, `regex` en dépendance de test), **frontend** (quatre
replis, un repli d'`InvoiceForm`, cinq tests et G13), plus un **`#[cfg(test)] mod tests` neuf** dans **kesh-db** (`vat_rates.rs`, sans code de
production — le module n'existe pas encore ; C-15-14-13 pour le lieu et le rayon du gate). Comptés en
modules métier de premier niveau, les replis de #569 en touchent plus de cinq (exercices, soldes de départ,
règlements, rapprochement, factures fournisseurs, facturation). **Dérogation assumée** : chaque changement
est un littéral de texte aligné sur un catalogue, sans logique ; la règle vise l'étendue d'un modèle
mental adversarial, qu'un remplacement de chaînes ne sollicite pas. Le découpage réel de la 15-14 est fait
ailleurs, sur une dépendance (C-15-14-1).

### Pièges

- **Apostrophes** : chaque clé garde son style (droite `'` ou typographique `’`) ; le repli Rust/TS est
  **octet pour octet** le catalogue — une apostrophe différente rend un vert du test frontend et un faux
  message côté API. ⚠️ **Divergence déjà présente** (validation P2, L-9) : le repli
  `routes/fiscal_years.rs:188` écrit `d'abord` (droite) quand fr-CH `:933` écrit `d’abord`
  (typographique) — garder « le style du repli » rendrait G9 rouge ; c'est le **catalogue** qui fait
  foi. Même règle pour le `jusqu’à` que l'AC 8 **ajoute** aux deux clés de soldes de départ : leurs valeurs
  actuelles ne portent **aucune** apostrophe (fr-CH `:964`, `:972`) ; le style typographique vient du
  tableau de l'AC 8, qui devient le catalogue, et les replis `opening_balances.rs:205`, `:406` et
  `+page.svelte:382` le recopient octet pour octet (validation P4, L-7 de la lentille R : « typographique
  au catalogue » était invérifiable tel qu'écrit).
- **Placeholders Fluent** : `{ $name }` dans `opening-balances-locked-first-year-closed` ; le repli Svelte
  `:382` porte la même forme.
- **PDF** : régénérer **les trois** (`make -B fr`), sous `mem-guard` (un `lualatex`/`xelatex` emballé a déjà
  emporté une session, § *Plafonds mémoire*). Vérifier que `\keshVersion` n'est pas touché (pas de release).
- **Garde LIFO, rappel** : la réouverture va du plus récent vers l'ancien — garde dans
  `crates/kesh-db/src/repositories/fiscal_years.rs` (`reopen`, `:1191` ; refus `:1232`, clé
  `FY_REOPEN_LIFO_BLOCKED_KEY` `:229`), traduite en message par `crates/kesh-api/src/routes/fiscal_years.rs:186`
  (validation P2, R-10 : le renvoi « `fiscal_years.rs:228` » ne disait pas quel fichier) ;
  « jusqu'à celui-ci » est vrai parce qu'on ne peut rouvrir un exercice que si aucun plus récent n'est clos.
- **`README.md:211`** reçoit des liens (AC 7) mais pas de réécriture de texte (AC 5).

### Références

- Issues : #539, #547, #488, #291, #458, #449, #432, #569, #321, #323 ; #459 ; #579, #324 (écartées).
- Registre : C-15-14-1 à C-15-14-8, C-15-14-11 à C-15-14-20, C-15-14-23, C-15-14-25 à C-15-14-27, C-15-14-31 ; C111, C122 (15-12a) ; C-15-12b-4 (B-1 = E-1).
- Fiches : `15-12a-cloture-dans-l-ordre.md` § *Hors périmètre* (liste d'origine des sites de #569).
- `CLAUDE.md` : § *Propagation post-patch*, § *Le prompt d'une passe doit NOMMER le manuel*, § *Inventorier
  les sites NON RÉSOLUS*, § *Recompter ses propres comptes rendus*.

### Ce que la validation P1 doit regarder

1. **Chaque défaut au code** : les constats « non corrigé » du tableau d'en-tête, et les faits sur lesquels
   les textes neufs reposent — trois plans choisis par `org_type`, aucune route d'import de plan ni de
   contacts, aucune tâche de fond d'inbox, #167 sous `[0.10.0]`, taux semés.
2. **Les inventaires par la valeur** : relancer chaque `grep` des AC ; chercher les sites que la fiche a
   manqués (autres locales, replis, `website/`, `docs/*.md`, manuels **et PDF aplatis**).
3. **Les textes des quatre locales** de l'AC 8 et de l'AC 10 : grammaire, collision, fidélité à la garde.
4. **Les tests** : chacun rougit-il vraiment sur sa mutation ? Une garde documentaire peut-elle passer à
   vide (chemin faux, fichier non lu) ?
5. **Les écartées** (C-15-14-2), surtout #324 et #579 : la raison tient-elle ?

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5 (`claude-opus-5-5`), worktree `kesh-15-14`, branche `story/15-14-lot-documentation-libelles`,
base `origin/main` = `245b91ee` (inchangé pendant tout le développement — `git fetch` au début et avant
la clôture : la 15-13b n'était pas mergée, aucun rebase). Sauvegarde : `backup/15-14-avant-dev`.

### Debug Log References

Journaux sous `/home/gcorbaz/devel/kesh-gate-logs/` : `15-14a-build-froid.log`, `15-14a-make-avant.log`,
`15-14a-make-t1.log`, `15-14a-make-final.log`, `15-14a-gate-backend.log`, `15-14a-vitest.log`,
`15-14a-build-front.log`, `15-14a-e2e.log`.

### Completion Notes List

**T0 — relevé.** Les dix-sept commandes d'inventaire, relancées telles qu'écrites (`LC_ALL=C.UTF-8`,
ensemble `E`), rendent **exactement** les comptes de la fiche sur `245b91ee` : AC 1 4 ; AC 2 (A) 21,
(B) 132 (38 texte, 84 commentaires au détecteur) ; AC 3 16 ; AC 5 25 ; AC 6 5 (`Unreleased` 2) ; AC 7 54 ;
AC 8 20 clés fr-CH, 153 lignes (106 de code), 8 Vitest ; AC 9 6 ; AC 10 20 en de-CH, 21 sur le dépôt.
Aucun écart à ventiler avant écriture. PDF de départ régénérés (`make -B fr`), puis contrôle « présent
avant » par `occ` : tous ≥ 1 (user : `doit d'abord le rouvrir` 3, `un administrateur doit d'abord rouvrir
l'exercice` 2, `2.5%` 2, `3.7%` 2, `Sterchi PME` 5, `KMU` 1, `Import d'un plan personnalisé` 2, `Import en
masse` 2, `dossier surveillé` 1, `Réglages` 4 ; admin : `Sterchi PME` 1, `n'est pas dans la v0.9.0` 1,
`Personnalisé CSV` 1, `Unreleased` 1 ; brochure : `Sterchi` 2, `dossier surveillé` 1, `KMU` 2). Le motif
`(Réglages → Facturation)` rendait 0 : le site `:908` s'écrit « (\emph{Réglages} → facturation) », en
minuscule — motif réécrit, contrôlé à 1 puis 0.

**T1-T4 — textes.** Manuel utilisateur (19 remplacements), manuel d'administration (7), brochure (3),
guide de démarrage (étape « Plan comptable » retirée, 7 → 6 étapes), `api-external.md` (2), README
(2 textes + 54 références d'issues en liens), `.env.example` (2), glossaire (1) ; catalogues fr-CH 8,
de-CH 23 clés + 1 commentaire, it-CH 9, en-CH 9 ; replis Rust 7 sites (6 clés) et 3 commentaires `#569`
retirés ; replis frontend 5. Contrôle du glossaire par la valeur (T2) : `grep -nF 'porte encore la
confusion' docs/i18n-glossaire.md` → aucune ligne ; `grep -nF '#323'` → ligne 102 (la ligne réécrite).
Collision de l'AC 10 : la commande rend 0 avant écriture, et après, seulement les six lignes attendues.
`fiscal-year-close-button` n'a qu'un repli, français (`settings/fiscal-years/+page.svelte:424`).

**PDF.** `make -B fr` deux fois (la première compilation après le retrait d'une sous-section signalait
« Label(s) may have changed » ; la seconde rend 0 avertissement de renvoi), aucun `??` dans les trois
PDF, `\keshVersion` inchangé (0.12.1). Contrôle `occ` final : les 21 anciens textes → 0 ; les 23 textes
neufs → ≥ 1 (`en commençant par le plus récent` → 7 au manuel utilisateur).

**Recompte après correction** (ventilé en C-15-14-42) : AC 1 3 (le doc-comment de G1), AC 2 (A) 14,
(B) 131 (+2 lignes de G3), AC 3 2, AC 5 21, AC 6 4 (les textes neufs dictés par l'AC 6 disent « jusqu'à
la v0.9.0 incluse » : l'« après : 2 » de la fiche était faux), `Unreleased` 0, AC 7 54 liens / 54
références, même numéro des deux côtés, AC 8 155 lignes (= 153 − 3 commentaires `#569` + 5 lignes de
G8), 20 clés fr-CH, AC 9 2 (lignes de G10), AC 10 8 en de-CH et 11 sur le dépôt (+2 lignes de G11).
Aucun site affiché ne reste à corriger.

**T5 — gardes.** G1 (`vat_rates.rs`, `mod tests` neuf, `#[test]` pur) ; G3, G8, G10, G11 (`loader.rs`,
valeurs brutes sans repli fr-CH — C-15-14-40) ; G2, G4, G5, G6, G7, G9, G12 (`textes_coherents.rs`,
sans base, chaque négatif adossé à un positif ; G2 exige que tout `.tex` de `docs/manual/fr` soit
gardé — C-15-14-41) ; G13 (Vitest, `describe` neuf). `regex = "1.10"` en `[dev-dependencies]` de
kesh-api et kesh-i18n (aucun paquet neuf au `Cargo.lock`, seules les deux lignes de dépendance). Les 7
sites Vitest de l'AC 8 (5 fichiers) basculés sur le marqueur « en commençant par le plus récent ». G12 :
normalisation rejouée sur les fichiers de `245b91ee` → 3 / 2 / 2 / 1, comme la fiche.

**T6 — mutations : 30, toutes ROUGES**, chacune appliquée par remplacement exact (occurrence unique
vérifiée), test visé lancé, fichier restauré par `git checkout` puis `touch` ; arbre propre ensuite :
G1 ×2 (`3.7\%` au manuel ; `380` → `370` dans `DEFAULT_SWISS_RATES`), G2 ×3, G3 ×2, G4 ×3 (« Sterchi
PME », sous-section d'import de contacts, « KMU » au guide), G5 ×2, G6 ×2, G7 ×2, G8 ×3 (fr-CH, de-CH,
exemption morte : `fiscal-year-reopen-button = Réactiver`), G9 ×2, G10, G11 ×2, G12 ×2, G13 ×3, et la
Vitest de l'AC 8 (ancien repli de `settlement-cancel-blocked.ts` → `settlement-cancel-blocked.test.ts`
rouge).

**T7.** CHANGELOG `[0.13.0] — Non publié` / `### Corrigé` : une entrée, les dix issues en liens. README
« Fonctionnalités » ne promet plus de surveillance de dossier (G5) ; feuille de route inchangée.

**T8 — gates, sur l'état final du code (`b22e7b71` ; `4803060e` ne porte que CHANGELOG, PDF et registre),
compilation à froid** (`CARGO_TARGET_DIR` du worktree vide au départ) :
- `cargo fmt --all -- --check` vert ; `cargo clippy --workspace --all-targets -D warnings` vert, 0 warning ;
- `scripts/test-fast.sh` (gate complet, exception `kesh-db`), base `kesh_1514a` neuve, migrée (75) et
  semée : **3056 / 3056** réussis, 4 ignorés ;
- `configuration_transmise` (lit `.env.example`) : 25 / 25, aussi inclus dans le gate complet ;
- frontend : `npm run check` 0 erreur, 27 avertissements (aucun sur un fichier touché) ;
  `lint-i18n-ownership` PASS ; `test:unit` **1140 / 1140** (114 fichiers ; 1139 + G13) ; `build` vert ;
- **E2E complet** au dernier commit de code, backend sur 3015, base `kesh_e2e_1514a` neuve, montage
  complet (SMTP, inbox, documents, backup ; `smtpConfigured: true`), lancé à 11:30 UTC : **245 réussis,
  9 échecs, 19 ignorés**. Les 9, nommément : 7 KF-029 (`mode-expert:26`, `:41`, `onboarding-path-b:65`,
  `:92`, `onboarding:57`, `:77`, `:150`) + 2 KF-045 matinales (`invoices:415`, `:439`, avant 12:00 UTC).
  Aucun hors liste, aucune pollution.

### File List

- `.env.example`
- `CHANGELOG.md`
- `Cargo.lock`
- `README.md`
- `crates/kesh-api/Cargo.toml`
- `crates/kesh-api/src/errors.rs`
- `crates/kesh-api/src/routes/fiscal_years.rs`
- `crates/kesh-api/src/routes/opening_balances.rs`
- `crates/kesh-api/tests/textes_coherents.rs` (nouveau)
- `crates/kesh-db/src/repositories/vat_rates.rs`
- `crates/kesh-i18n/Cargo.toml`
- `crates/kesh-i18n/locales/{de-CH,en-CH,fr-CH,it-CH}/messages.ftl`
- `crates/kesh-i18n/src/loader.rs`
- `docs/api-external.md`
- `docs/i18n-glossaire.md`
- `docs/manual/fr/{admin-manual,user-manual,marketing-brochure}.{tex,pdf}`
- `docs/user-guide/fr/getting-started.md`
- `frontend/src/lib/components/invoices/InvoiceForm.svelte`
- `frontend/src/lib/features/invoices/InvoiceSettlements.test.ts`
- `frontend/src/lib/features/reconciliation/CancelReconciliationDialog.test.ts`
- `frontend/src/lib/features/reconciliation/reconciliation-cancel.test.ts`
- `frontend/src/lib/features/reconciliation/reconciliation-cancel.ts`
- `frontend/src/lib/features/supplier-invoices/invoice-cancel.ts`
- `frontend/src/lib/shared/i18n-repli-divergent-actif.test.ts`
- `frontend/src/lib/shared/utils/settlement-cancel-blocked.test.ts`
- `frontend/src/lib/shared/utils/settlement-cancel-blocked.ts`
- `frontend/src/routes/(app)/invoices/[id]/invoice-settlements-page.test.ts`
- `frontend/src/routes/(app)/settings/opening-balances/+page.svelte`
- `_bmad-output/implementation-artifacts/15-14a-manuels-et-libelles.md`,
  `epic-15-choix-autonomes.md` (C-15-14-38 à 42), `sprint-status.yaml`

## Change Log

- 2026-10-09 — Spécification (Opus 5.5), sur `dc4bc58b`. Choix C-15-14-1 à C-15-14-8.
- 2026-10-09 — **Validation P1** (Sonnet ×2 : lentilles R — auditeur d'acceptation — et F — adversaire
  plein périmètre ; rapports `kesh-gate-logs/15-14-validate-p1-{R,F}.md`). Pour cette fiche : 0 CRITICAL,
  0 HIGH, **6 MEDIUM distincts**, 7 LOW distincts (décompte des deux fiches : index, § Change Log).
  Branche rebasée sur `bcded0c8` (15-13a mergée) : manuel d'administration, `.env.example` et CHANGELOG
  réalignés ; le reste n'avait pas bougé. Chaque finding vérifié au code avant correction :
  - **MEDIUM R1 = F-1** — `user-manual.tex:1208-1209` (« doit d'abord \textbf{rouvrir} l'exercice ») hors
    inventaire de #569 : **corrigé**, ajouté à l'AC 8 et au T1 ; « quatre phrases » → **cinq** (en-tête,
    index, C-15-14-16).
  - **MEDIUM R3 ≈ F-11** — inventaire « Réglages » sensible à la casse : **corrigé** — deux sites étaient des
    renvois au menu (fr-CH `error-invoice-pdf-header-overflow` + repli `errors.rs:1807` ; `.env.example:305`,
    hors périmètre du grep), les autres « réglages » minuscules sont le nom commun et entrent à l'inventaire
    des non résolus (C-15-14-11). T2, T3, T9 étendus.
  - **MEDIUM R4** — glossaire `docs/i18n-glossaire.md:102` rendu faux par l'AC 10 : **corrigé** (AC 10, grep
    étendu à `docs/`).
  - **MEDIUM R5 ≈ F-2** — T8 sans verbe par locale, de-CH à cinq formes, exemptions incomplètes : **corrigé**
    par un autre dessin — domaine pris en fr-CH (20 clés), contrôlé dans les quatre locales, exemptions par
    clé, partition 6 + 4 + 10 recomptée ; mutation de-CH ajoutée (C-15-14-14). « wieder eröffnen » →
    « wieder öffnen ».
  - **MEDIUM F-4** — `.env.example:197` « dossier surveillé » : **corrigé** (AC 5, T5 étendu) ; R7 (« scruté
    à l'import ») tranché : exact, conservé (C-15-14-12) ; redite de `:1524` retirée.
  - **MEDIUM F-6 ≈ R6** — `vat_rates.rs` n'a pas de `mod tests` : **corrigé** — module neuf `#[cfg(test)]`,
    `#[test]` pur, constante non exposée ; exception `kesh-db` du gate (C-15-14-13).
  - **LOW** : R11 = F-7 (T6 forme LaTeX) corrigé ; R12 (`:182` conditionnel contredit par T4) corrigé — `:182`
    réécrit sans condition, texte identique à la 15-7b1 ; F-12 (`invoice-settlements-page.test.ts:182`)
    corrigé ; F-13 (de-CH `settings-fiscal-years-link`) corrigé ; R13 ≈ F-8 (conflits de PDF, dépendance
    « aucune ») corrigé au T0 et à l'index ; R7 corrigé (voir F-4) ; **F-14 réfuté** — `plan-comptable.png`
    n'existe pas (`docs/manual/fr/screenshots/` ne contient que `_placeholder.png`), seule la légende porte le
    texte, déjà corrigée.
  - **Ajout de l'orchestrateur de remédiation** (LOW) : T9 compare les replis Rust **après normalisation des
    continuations** `\` — le repli de `error-invoice-pdf-header-overflow` s'écrit sur cinq lignes.
  - Propagation : valeurs grepées sur le dépôt (`rouvr`, `[Rr]églages`, `surveill|scrut`, `46`, `1931|1947|1361`,
    `1acbf918`, `KF-041`, `schliessen`) ; décomptes recomptés à la source (20 clés fr-CH du domaine de T8 ;
    22 sites « réglages » minuscules assumés ; tâches T0-T8 : 9 ; tests T1-T11 + Vitest ; AC 1-11 : 11 ; issues : 10
    fermées + refs #459).
  - Signal D5 : non levé — première passe ; défauts distincts, aucun né d'une remédiation.
- 2026-10-09 — **Validation P2** (Opus ×2 : lentilles R et F ; rapports `kesh-gate-logs/15-14-validate-p2-{R,F}.md`).
  Pour cette fiche : 0 CRITICAL, 0 HIGH, **5 MEDIUM distincts** (dont F-1, partagé avec la 15-14b), **8 LOW
  distincts** (dont R-14, partagé) ; décompte des deux fiches : index, § Change Log. Chaque finding vérifié au
  code (`grep -nF`, commandes exécutées sur `bcded0c8`) avant correction ; **aucun réfuté**.
  - **MEDIUM F-1** — contrôle aplati des PDF à vide (apostrophes `’`, traits d'union coupés) : **corrigé** —
    fonction `occ` normalisant PDF et motif, contrôle « présent avant » obligatoire, valeurs vérifiées sur les
    PDF de `bcded0c8` (convention des AC ; T0, T1).
  - **MEDIUM F-4** — `docs/user-guide/fr/getting-started.md:29` (KMU, Verein, import CSV de plan) : **corrigé**
    — `docs/user-guide/` dans le périmètre de tous les inventaires, ligne retirée et étape 1 complétée (AC 3),
    G4 étendu au guide (C-15-14-17).
  - **MEDIUM R-1 = F-5** — « le seul autre site de-CH » faux : **corrigé** — inventaire de 20 lignes
    partitionné, six participes de-CH corrigés (« abgeschlossen »), G11 étendu (C-15-14-18).
  - **MEDIUM R-2 = F-6** — inventaire « réglages » : 28 lignes, **26** assumées et non 22 (`:364`, `:380`,
    `:394` manquaient) ; `:965` tranché assumé : **corrigé** (AC 2, C-15-14-17 rectifie C-15-14-11).
  - **MEDIUM F-9 (≈ R-6, LOW)** — les cinq phrases du manuel et `api-external.md:330` sans garde : **corrigé** —
    G12 (`le_manuel_dit_l_ordre_de_reouverture`), avec inventaire documentaire de 25 lignes (6 + 5 + 14).
  - **LOW** : R-7 (Vitest : 7 sites dans 5 fichiers, six assertions et une donnée de mock ; 8 lignes au
    grep) corrigé ; R-8 = L-4 (cinq clés à repli Rust, sept sites : table de G9) corrigé ; R-9 = L-2
    (`admin-manual.tex:2516`) corrigé ; R-10 (renvoi LIFO ambigu : chemin complet du repository) corrigé ;
    R-14 (tests renommés G1-G12) corrigé ; L-3 (deux verbes de-CH) tranché, assumé (C-15-14-20) ; L-8
    (`vat_rates.rs` modifié par la 15-7b1) déclaré au T0 et à l'index ; L-9 (apostrophe du repli
    `fiscal_years.rs:188`) écrit aux Pièges. Dépendance envers la 15-7b1 déclarée pour `:182` (F-7/L-8,
    C-15-14-19), repli à texte identique conservé.
  - **Propagation** : valeurs grepées sur le dépôt et les fiches — `\b22\b` (réglages), `2405`, `228`,
    `six assertions`, `seul autre site`, `consigne 8`, `T1`-`T11` des tests ; chaque commande d'inventaire
    des AC **exécutée** et son compte écrit (AC 1 : 2 ; AC 2 : 17 et 28 ; AC 3 : 18 ; AC 5 : 13 ; AC 6 : 3 ;
    AC 7 : 54 ; AC 8 : 20 clés, 25 lignes, 8 lignes Vitest ; AC 9 : 6 ; AC 10 : 20). Recomptes : AC 1-11 : 11 ;
    tâches T0-T8 : 9 ; tests G1-G12 : 12 + Vitest ; issues : 10 fermées + refs #459.
  - **Signal D5 levé** (MEDIUM → MEDIUM ; F-5 et F-6, ici, sont nés de la remédiation P1) : pas de nouveau
    découpage (C-15-14-23), déclaré à l'index.
- 2026-10-09 — **Validation P3** (Sonnet ×2 : lentilles R et F ; rapports `kesh-gate-logs/15-14-validate-p3-{R,F}.md`).
  Pour cette fiche : 0 CRITICAL, 0 HIGH, **2 MEDIUM distincts** (R-1, F-1 ; R-2, de planification, est
  porté à l'index), **6 LOW distincts** (dont L-1 et F-6, partagés ; F-4, de la 15-14b, appliqué ici par
  extension à G12) ; décompte des deux fiches : index, §
  Change Log. Chaque finding vérifié au code avant correction ; **aucun réfuté**. Branche rebasée sur
  `245b91ee` (15-7b1 mergée) : comptes relevés sur cet état, numéros de ligne laissés sur `bcded0c8` avec la
  table des décalages de la 15-7b1 (convention ; C-15-14-25).
  - **MEDIUM R-1** — « gardé par `i18n-repli-divergent-actif.test.ts` » faux : la garde ne retient que les
    clés à deux replis distincts (`grep -nF "parTexte.size > 1"` → ligne 141), et les cinq clés visées ont un
    seul site d'appel chacune (vérifié par `grep -rln` sur `frontend/src`). **Corrigé** : G13, `describe` neuf
    du même fichier, table fermée de cinq clés, ensemble des replis = `[valeur fr-CH]`, trois mutations
    (C-15-14-26) ; l'affirmation retirée de l'AC 2, de l'AC 8 et des mutations.
  - **MEDIUM F-1** — commande (B) bornée à quatre fichiers ; `README.md:34` (« saisis une fois dans les
    réglages ») lui échappait : **corrigé** — toutes les commandes portent sur le dépôt suivi moins `E`
    (C-15-14-25) ; (A) 21 = 7 + 14, (B) 132 = 3 + 129 (38 lignes de texte, 94 de code dont 86 commentaires) ;
    `README.md:34` corrigé, G2 étendu au README. Les autres AC relancés de même : AC 1 4 (2 + 2), AC 3 16
    (14 + 2 ; `onboarding-org-pme = KMU` assumé), AC 5 25 (4 + 21), AC 6 5 (3 + 2), AC 8 153 (21 + 26 + 106 :
    aucun repli hors liste ne prescrit la réouverture sans l'ordre), AC 9 6, AC 10 21 (20 + le glossaire).
  - **LOW** : L-1 (`regex` absent de kesh-api et kesh-i18n) corrigé — `[dev-dependencies]`, C-15-14-27 ;
    L-2 (AC 3 : 18 → 17 après la 15-7b1) dépassé — 16 sur le dépôt entier ; L-3 = F-3 (`occ` dépendante de
    la locale) corrigé — `export LC_ALL=C.UTF-8` en tête, et la limite du saut de page écrite comme angle
    mort (aucun remède robuste ; G12 couvre #569) ; L-7 (de/it/en disaient déjà le menu) écrit comme
    extension de périmètre voulue ; F-4 (normalisation à un niveau) appliqué à G12 — dépliage jusqu'à
    stabilité, comptes « présent avant » inchangés (3 / 2 / 2 / 1, rejoués) ; F-6 (deux « G5 ») corrigé —
    G13 ici, G14-G18 à la 15-14b (C-15-14-31) ; F-7 (CHANGELOG et PRD au périmètre hétérogène) corrigé par
    `E`, chaque exclusion justifiée.
  - **Trouvé par la remédiation** : le PDF de `bcded0c8` était en retard sur son `.tex` (`un administrateur
    doit d'abord rouvrir l'exercice` → 1 sur ce PDF, 2 sur celui de `245b91ee`) — le T0 régénère désormais
    les PDF avant le contrôle « présent avant ».
  - **Propagation** : valeurs grepées sur les trois fiches et le registre — `\b(18|28|13|25|34|26)\b` des
    anciens comptes, `en cours de merge`, `#583`, `Indépendante`, `i18n-repli-divergent-actif`, `G1`-`G5` de
    la 15-14b, `awk` ; chaque commande des AC **exécutée telle qu'écrite** (copiée depuis la fiche) sur
    `245b91ee`. Recomptes : AC 1-11 : 11 ; tâches T0-T8 : 9 ; tests G1-G13 : 13 + Vitest ; issues : 10
    fermées + refs #459.
  - **Signal D5** : levé et déclaré à l'index.
- 2026-10-09 — **Validation P4** (Opus ×2 : lentilles R et F ; rapports `kesh-gate-logs/15-14-validate-p4-{R,F}.md`).
  Pour cette fiche : 0 CRITICAL, 0 HIGH, **0 MEDIUM** ; LOW : R L-2, L-5, L-7, L-8 et F L-2. Tous les comptes
  de `b02e9af7` se recomptent à l'identique (AC 1 à AC 10, rejoués par les deux lentilles). **Validation
  close** (C-15-14-32) : critère d'arrêt du `CLAUDE.md` atteint pour la 15-14a ; trend du lot 8 → 9 → 4 → 3
  MEDIUM, les trois de la P4 tous sur la 15-14b. Statut `ready-for-dev` (inchangé).
  - **LOW** : R L-7 (« `jusqu’à` … typographique au catalogue », invérifiable : les valeurs actuelles n'ont
    pas d'apostrophe — `fr-CH:964`, `:972`) corrigé aux Pièges ; R L-8 (glossaire sans contrôle) corrigé —
    `grep -nF` par la valeur au T2 ; F L-2 (`make fr` ne rebâtit pas un PDF de même date que son `.tex`)
    corrigé — `make -B fr` au T0, au T1, aux Pièges et dans la convention ; R L-2 (« Paramètres →
    Facturation » est un titre d'écran, non un chemin de menu) **hors périmètre**, écrit à l'AC 2 : issue
    #585 (P4) ouverte par l'orchestrateur ; R L-5 (#459 non prévenue) : écrit à l'AC 5, #459 commentée par
    l'orchestrateur. R L-3 (`about.html:96`) porte sur l'AC 3 de la 15-14b, corrigé là.
  - **Propagation** : `make fr`, `make -C docs/manual fr`, `typographique au catalogue` grepés dans les
    deux fiches : ne restent que les Change Logs antérieurs. Recomptes : AC 1-11 : 11 ; tâches T0-T8 : 9 ;
    gardes G1-G13 : 13 ; issues : 10 fermées + refs #459.
- 2026-10-09 — **Développement** (Opus 5.5, `bmad-dev-story`), sur `245b91ee`. T0-T8 faits ; inventaires du
  T0 conformes à la fiche ; 30 mutations rouges ; gates sur l'état final, compilation à froid : backend
  3056/3056, Vitest 1140/1140, E2E 245 / 9 attendus (7 KF-029 + 2 KF-045). Choix C-15-14-38 à 42 (tableau
  des plans, ordre des propositions de-CH/it-CH/en-CH, valeurs brutes des catalogues, garde de tout `.tex`,
  comptes « après » ventilés — dont l'« AC 6 : 2 » de la fiche, faux). Statut `review`.

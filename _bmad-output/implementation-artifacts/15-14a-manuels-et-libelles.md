# Story 15-14a : Manuels et libellés — dix défauts P3/P4 qui disent autre chose que le code

Status: ready-for-dev

<!-- Spécifiée le 2026-10-09 sur origin/main = dc4bc58b (worktree kesh-15-14). Sous-story de la 15-14
     (lot de défauts de documentation et de libellés, découpé d'emblée — C-15-14-1). Choix C-15-14-1 à 8. -->

## Story

En tant qu'utilisateur de Kesh qui suit le manuel ou lit un message d'erreur,
je veux que le texte décrive ce que le logiciel fait réellement — taux, menus, plans comptables,
imports, versions, ordre de réouverture, vocabulaire —,
afin de ne pas chercher un écran qui n'existe pas ni tenter un geste que Kesh refusera.

**Issues fermées** (une ligne `closes` par issue dans le titre ou le corps de la PR — § *Commits qui
adressent une issue* du `CLAUDE.md`) :

| Issue | Prio. | Labels | Objet | État au code (`dc4bc58b`) |
|---|---|---|---|---|
| #539 | P3 | bug, documentation | taux de TVA du manuel (2,5 / 3,7 %) | **non corrigé** — `user-manual.tex:875`, `:1928` |
| #547 | P4 | bug | « Réglages » au lieu de « Paramètres » | **non corrigé** — 4 sites au manuel **+ 1 message** (propagation) |
| #488 | P3 | bug, documentation | « deux plans pré-configurés (Sterchi PME, KMU) » | **non corrigé** — 12 sites dans 3 documents |
| #291 | P3 | documentation | « Contacts → Import CSV » inexistant | **non corrigé** — `user-manual.tex:857-859` |
| #458 | P3 | documentation | « dossier surveillé » | **non corrigé** — 3 sites (+1 historique assumé) |
| #449 | P3 | bug | faille KF-036 annoncée ouverte | **non corrigé** — `api-external.md:484` **+ 2 sites au manuel d'administration** |
| #432 | P4 | bug | `[#NNN]` du README | **non corrigé** — 54 références, lignes 211-223 |
| #569 | P3 | bug | « rouvrir l'exercice » sans l'ordre LIFO | **non corrigé** — 6 clés ×4, 6 replis Rust, 4 replis frontend, 4 phrases du manuel, 1 ligne d'API |
| #321 | P4 | bug | `MwSt` contre `MWST` (de-CH) | **non corrigé** — 5 valeurs + 1 commentaire |
| #323 | P3 | known-failure | « Clôturer » = « Fermer » dans les cibles | **non corrigé** — 2 clés ×3 + titre/corps de-CH |

`refs #459` (automatisation de l'import, que #458 renvoie). **Huit** de ces issues portent `bug` ou
`known-failure` et comptent dans le décompte de l'engagement 1 (#539, #547, #488, #449, #432, #569,
#321, #323) ; #291 et #458 ne portent que `documentation`.

**Hors de cette story** (C-15-14-2) : #579 (15-6c), #324 (prémisse fausse, arbitrage de Guy), #469,
#339, #504 ; et la 15-14b (#575, #554, #127), qui suit celle-ci.

## Acceptance Criteria

> Convention : « relevé sur `dc4bc58b` » — les numéros de ligne **se périment** ; le développeur
> retrouve chaque site **par la valeur** (commande donnée), jamais par le numéro. Pour chaque
> manuel modifié : **régénérer le PDF** (`scripts/mem-guard.sh make -C docs/manual fr`) et le
> **contrôler aplati** — `pdftotext f.pdf - | tr '\n' ' ' | tr -s ' ' | grep -F '<texte>'` — sur
> le texte neuf (présent) et sur l'ancien (absent).

### AC 1 — Les taux de TVA du manuel sont ceux que Kesh pose (#539)

Kesh pose 0 %, 2,6 %, 3,8 % et 8,1 % (`DEFAULT_SWISS_RATES`, `crates/kesh-db/src/repositories/vat_rates.rs:281`,
`valid_from = 2024-01-01`).

- `user-manual.tex:875` devient :
  `\item \textbf{Taux TVA} : 0\%, 2.6\%, 3.8\%, 8.1\% --- les taux suisses en vigueur depuis le 1\textsuperscript{er} janvier 2024, que Kesh pose à la création de la société (\emph{Paramètres} → \emph{Taux de TVA}).`
  (« au moment de la rédaction » disparaît : c'est lui qui a laissé la phrase se périmer.)
- `user-manual.tex:1928` : « ventilée par taux (8.1\%, 3.8\%, 2.6\%, 0\%) ».
- Sites par la valeur : `grep -rnE '2[.,]5 ?\\?%|3[.,]7 ?\\?%|7[.,]7 ?\\?%' docs/manual website README.md docs/*.md`
  → **zéro** après correction (sur `dc4bc58b` : les deux sites ci-dessus, rien ailleurs).
- **Test** (T1) : `le_manuel_cite_les_taux_poses_par_kesh`, dans le `mod tests` de `vat_rates.rs` — lit
  `docs/manual/fr/user-manual.tex` (`env!("CARGO_MANIFEST_DIR")/../../docs/…`) ; pour chaque taux de
  `DEFAULT_SWISS_RATES`, sa forme `{entier}.{décimale}\%` (`810` → `8.1\%`, `0` → `0\%`) figure sur la
  ligne `\textbf{Taux TVA}` ; aucune des formes `2.5\%`, `3.7\%`, `7.7\%` n'apparaît dans un
  `docs/manual/fr/*.tex`. Mutations : réécrire `3.7\%` au manuel → rouge ; changer `380` en `370` dans
  la constante → rouge (sur le manuel corrigé).

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
  - repli `frontend/src/lib/components/invoices/InvoiceForm.svelte:768` **identique** au catalogue fr-CH
    (la garde `i18n-repli-divergent-actif.test.ts` l'impose déjà).
- Sites par la valeur : `grep -rn 'Réglages' docs/manual frontend/src crates/kesh-i18n/locales crates/kesh-api/src website README.md docs/*.md`.
  **Inventaire des sites non résolus, assumés** : `README.md:213` (feuille de route publiée) ; les noms
  d'entité et d'action du journal d'audit (`audit-log-entity-company-*-settings`,
  `audit-log-action-company-*-settings-*`) — des noms d'objet, non des renvois au menu ; les commentaires
  de code (`dunning/+page.svelte:2`, `dunning.types.ts:34`, `dunning.api.ts:57`, `write-off.ts:75`) ;
  `frontend/tests/e2e/dunning.spec.ts:6,26` (titres de test). Tout autre site trouvé se corrige ou
  s'ajoute à cette liste, avec sa raison.
- **Tests** : T2 (`aucun_renvoi_au_menu_reglages`, garde documentaire : aucun `\emph{Réglages}` ni
  `Réglages et` dans `docs/manual/fr/*.tex`) ; T3 (`le_message_du_compte_de_produit_renvoie_a_l_ecran_reel`,
  kesh-i18n : pour chaque locale, la valeur de `invoice-default-revenue-account-unusable` contient
  `settings-invoicing-title` de la même locale, ` — ` remplacé par ` → `). Mutations : remettre
  « dans les Réglages » au catalogue fr → T3 rouge ; remettre `\emph{Réglages}` au manuel → T2 rouge.

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
  `admin-manual.tex:1361-1372` (tableau « Choix du plan comptable ») réécrit sur les trois plans réels et
  leur choix par le type d'organisation, **sans** la ligne « Personnalisé CSV » ; le `keshtip` qui suit
  reste juste (le plan se modifie après création) — à relire.
- `marketing-brochure.tex:260` : « Plan comptable suisse PME, adapté à votre forme juridique. » ;
  `:385` : « Plan comptable suisse (PME, indépendant, association) avec saisie d'écritures en partie double. »
- **Assumé, non résolu ici** : `user-manual.tex:182` (« Une company fictive avec un plan comptable Sterchi
  PME ») — la **15-7b1** réécrit ce bloc (« Le plan comptable PME, dans la langue de l'installation »,
  vérifié sur `origin/story/15-7b1-trace-demonstration`). Si la 15-7b1 est mergée avant, rien à faire ; sinon
  corriger ici la seule ligne 182 et le signaler à l'orchestrateur (conflit de rebase attendu, trivial).
- **Liens externes conservés** : `admin-manual.tex:2405`, `user-manual.tex:2461` (URL `kmu.admin.ch`).
- Sites par la valeur : `grep -rn 'Sterchi\|KMU\|pré-configur\|Import CSV' docs/manual/fr/*.tex README.md website/*.html`.
- **Test** T4 (`plans_comptables_reels`) : aucun `Sterchi`, aucun `KMU` hors URL dans `docs/manual/fr/*.tex`
  ni `README.md` ; aucun `\emph{Import CSV}` (forme des deux imports fictifs — l'import bancaire s'écrit
  `\emph{Nouvel import CSV}` et reste permis). Mutation : remettre « Sterchi PME » à `:308` → rouge.

### AC 4 — Le manuel ne promet plus d'import de contacts (#291, C-15-14-3)

- `user-manual.tex:857-859` (`\subsubsection{Import en masse}` et sa phrase) **retirés**. Vérifié sur
  `dc4bc58b` : aucune route `/api/v1/contacts/import`, aucun bouton d'import à
  `frontend/src/routes/(app)/contacts/+page.svelte`.
- Couvert par T4 (`\emph{Import CSV}` interdit). Mutation : remettre la sous-section → rouge.

### AC 5 — L'import de factures se dit déclenché à la main (#458, refs #459)

Aucun processus ne surveille `KESH_INBOX_DIR` (vérifié : aucune tâche de fond dans `kesh-api` ;
`inbox_import.rs` n'est appelé que par `POST /api/v1/inbox-import`).

- `user-manual.tex:1524` : « … déposées dans le dossier d'import (voir le manuel administrateur pour sa
  configuration). L'import ne se déclenche pas tout seul : il se lance à la main, depuis \emph{Quotidien}
  → \emph{Importer des factures}. »
- `marketing-brochure.tex:396` : « Import de factures fournisseurs déposées dans un dossier, avec décodage
  du QR-facture côté serveur. »
- `README.md:43` : « dépôt de factures … dans un dossier d'import, import lancé depuis l'écran, décodage … ».
- **Assumé** : `README.md:211` (feuille de route v0.4.0 publiée — on ne réécrit pas l'historique ; seule
  la forme de ses références d'issues change, AC 7). Les « Surveiller les logs / CVE » du manuel
  d'administration sont d'un autre sens.
- Sites par la valeur : `grep -rn -i 'surveill' docs/manual README.md website docs/*.md`.
- **Test** T5 (`aucun_dossier_surveille`) : la chaîne `dossier surveillé` n'apparaît dans aucun
  `docs/manual/fr/*.tex`, ni dans `README.md` hors de la ligne de feuille de route `| v0.4.0 |`.

### AC 6 — La correction de la KF-036 se dit livrée, dans la version qui l'a livrée (#449)

Le correctif (#167) est sous `## [0.10.0] — 2026-08-19` du CHANGELOG (`CHANGELOG.md:321`, section
`:295-324`).

- `docs/api-external.md:484` devient : « ⚠️ **Dans quelle version ?** Cette fermeture est livrée depuis la
  **v0.10.0** (entrée [#167](https://github.com/guycorbaz/kesh/issues/167) du [CHANGELOG](../CHANGELOG.md)).
  Jusqu'à la v0.9.0 incluse, une clé `read-write` créée par un Administrateur atteint les routes
  d'administration : sur une telle version, traitez une clé d'origine Administrateur comme un secret
  d'administrateur, et mettez Kesh à jour. »
- `admin-manual.tex:1931` : la phrase « \textbf{Attention à la version} : cette fermeture n'est pas dans la
  v0.9.0 --- voir l'encadré … » devient « Cette fermeture est livrée depuis la v0.10.0 --- voir l'encadré
  \emph{« Dans quelle version ? »} plus bas. »
- `admin-manual.tex:1947` (encadré) : titre « Dans quelle version ? Depuis la v0.10.0. » ; corps : la
  faille existait jusqu'à la v0.9.0 incluse ; le correctif est livré par la v0.10.0 (entrée \#167 du
  \keshcommand{CHANGELOG.md}) ; sur une version antérieure, la consigne de prudence. **Plus aucun renvoi à
  une section `[Unreleased]`** (elle se renomme à chaque release — c'est ce qui a périmé le texte).
- Sites par la valeur : `grep -rnF 'Unreleased' docs/api-external.md docs/manual README.md website` et
  `grep -rn 'v0\.9\.0' docs/api-external.md docs/manual` (les lignes de feuille de route du README et de
  `website/roadmap.html` restent).
- **Test** T6 (`la_faille_kf036_n_est_pas_annoncee_ouverte`) : ni `Unreleased`, ni `n'est **pas** dans la
  v0.9.0`, ni `Pas dans celle que décrit ce manuel` dans `docs/api-external.md` et
  `docs/manual/fr/admin-manual.tex` ; `v0.10.0` figure dans le paragraphe qui suit « Dans quelle version ».

### AC 7 — Toute référence d'issue du README est un lien (#432, C-15-14-6)

- Les 54 références `#NNN` du README (lignes 211-223 sur `dc4bc58b` : 32 `[#NNN]`, 22 nues) s'écrivent
  `[#NNN](https://github.com/guycorbaz/kesh/issues/NNN)`. Contenu des lignes inchangé par ailleurs.
- Recompte à la source avant et après : `grep -oE '#[0-9]{2,4}\b' README.md | wc -l` (54 avant) ;
  `grep -oE '\[#[0-9]+\]\(https://github.com/guycorbaz/kesh/issues/[0-9]+\)' README.md | wc -l` (54 après).
- **Test** T7 (`references_d_issues_du_readme_sont_des_liens`) : toute occurrence de `#\d+` du README
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

- de-CH : « … muss ein Administrator die abgeschlossenen Geschäftsjahre bis zu diesem wieder eröffnen,
  beginnend mit dem neuesten » ; reopen-blocked : « Wiedereröffnung nicht möglich: Ein späteres
  Geschäftsjahr ist abgeschlossen; eröffnen Sie zuerst die abgeschlossenen Geschäftsjahre wieder,
  beginnend mit dem neuesten. »
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
récent ») : `settlement-cancel-blocked.test.ts:34`, `invoice-settlements-page.test.ts:173`,
`CancelReconciliationDialog.test.ts:91`, `reconciliation-cancel.test.ts:25,61`,
`InvoiceSettlements.test.ts:79`.

**Documentation** :

- `user-manual.tex:1456`, `:1478` : « (un administrateur doit d'abord rouvrir les exercices clôturés
  jusqu'à celui-ci, en commençant par le plus récent) » ; `:1773` : « un administrateur doit d'abord
  rouvrir les exercices clôturés jusqu'à celui-ci, en commençant par le plus récent. » ; `:2316-2317` :
  « un administrateur doit d'abord rouvrir les exercices clôturés jusqu'à celui de la pièce, en commençant
  par le plus récent. » `:2311` (« aucune raison de rouvrir l'exercice ») **décrit**, il reste. `:717`,
  `:722` (procédure de réouverture, qui dit déjà la garde d'ordre plus bas) restent.
- `docs/api-external.md:330` : « Règlement d'un exercice **clos** — un administrateur doit rouvrir les
  exercices clôturés jusqu'à celui-ci, en commençant par le plus récent ».

**Inventaire des sites non résolus** (par la valeur, `grep -rnE "rouvr|wieder ?(er)?öffn|wiedereröffn|riapr|reopen" crates/kesh-i18n/locales`),
**conservés avec leur raison** — le test T8 les tient en liste fermée :

- `journal-entries-modify-blocked-later-fiscal-year-closed`, `error-later-fiscal-year-closed`,
  `error-fiscal-year-create-later-closed`, `fiscal-year-out-of-order-warning` : portent déjà le marqueur ;
  la clause de contre-passation des deux premières est examinée et conservée (C-15-14-4, renvoi B-1 = E-1
  de la 15-12b) ;
- `fiscal-year-reopen-blocked-later-closed` : nomme l'exercice à rouvrir (le plus récent) ;
- `fiscal-year-close-confirmation-body`, `fiscal-year-reopen-confirmation-body` : décrivent, ne prescrivent pas ;
- fr-CH `error-reminder-amounts-changed` (« rouvrez l'aperçu ») : autre objet ;
- tout autre site qu'un recompte révèle (clés d'audit `…reopened`, libellés de bouton `Réouvrir`) s'ajoute
  à la liste avec sa raison, ou se corrige.

**Test** T8 (`les_prescriptions_de_reouverture_disent_l_ordre`, kesh-i18n, `loader.rs` `mod tests`) :
pour chaque locale, toute valeur qui matche le verbe de réouverture de la locale contient le marqueur
d'ordre de la locale (`en commençant par le plus récent` / `beginnend mit dem neuesten` /
`cominciando dal più recente` / `starting with the most recent`), **sauf** les clés de la liste fermée ;
chaque clé exemptée doit **encore** matcher le verbe (sinon l'exemption est morte → rouge) ; les six clés
de l'AC sont nommées et doivent porter le marqueur (anti-test-muet). T9 (garde des replis Rust,
`textes_coherents.rs`) : chacun des trois fichiers Rust contient la valeur fr-CH exacte de ses clés et
aucune des chaînes `rouvrir l'exercice pour`, `rouvrez-le`. Mutations : remettre l'ancienne valeur fr-CH
de `error-fiscal-year-reopen-blocked` → T8 rouge ; remettre l'ancien repli d'`errors.rs:3085` → T9 rouge
(et le repli frontend → `i18n-repli-divergent-actif.test.ts` rouge).

### AC 9 — `MWST` partout en de-CH (#321)

- de-CH : `invoice-line-vat-rate = MWST %` ; `invoice-error-vat-invalid = MWST-Satz nicht erlaubt. …` ;
  `error-invoice-too-many-lines-for-pdf` : « mit der MWST-Zusammenfassung » ; `reports-vat = MWST` ;
  `reports-vat-column-vat-due = Geschuldete MWST` ; commentaire `# MWST-Bericht (Story 11-2)`.
  Relevé sur `dc4bc58b` : `de-CH/messages.ftl:565`, `:608`, `:680`, `:1293`, `:1294`, `:1297`.
- **Ne pas propager** aux autres locales (IVA, TVA, VAT sont justes — avertissement de l'issue).
- Sites par la valeur : `grep -rnE '\bMwSt\b' crates frontend/src docs website` (replis compris).
- **Test** T10 (`glossaire_mwst`, kesh-i18n) : aucune valeur de-CH ne matche `\bMwSt\b` ; au moins une
  valeur contient `MWST` (anti-test-muet). Mutation : remettre `reports-vat = MwSt` → rouge.

### AC 10 — Clôturer un exercice n'emprunte plus le verbe des panneaux (#323, C-15-14-7)

| Clé | de-CH | it-CH | en-CH |
|---|---|---|---|
| `fiscal-year-close-button` | Abschliessen | Chiudi l’esercizio | Close fiscal year |
| `fiscal-year-close-confirmation-action` | Abschliessen | Chiudi l’esercizio | Close fiscal year |
| `fiscal-year-close-confirmation-title` | Geschäftsjahr abschliessen? | (inchangé) | (inchangé) |
| `fiscal-year-close-confirmation-body` | « … das Geschäftsjahr „{ $name }“ abzuschliessen. Solange es abgeschlossen bleibt, … » | (inchangé) | (inchangé) |

- fr-CH inchangé (« Clôturer ») : la suite E2E, en français, ne voit rien.
- Vérifier la collision avant d'écrire : `grep -nE '= (Abschliessen|Chiudi l’esercizio|Close fiscal year)$' crates/kesh-i18n/locales/*/messages.ftl`
  doit ne rendre que les clés ci-dessus.
- Vérifier que le bouton n'a pas de repli dans une autre langue que le français et qu'aucun test ne
  fige `Schliessen`/`Chiudi`/`Close` pour ces clés (`grep -rn 'fiscal-year-close-button' frontend/src crates`).
- **Test** T11 (`la_cloture_d_exercice_ne_parle_pas_comme_un_panneau`, kesh-i18n) : pour de-CH, it-CH,
  en-CH, les valeurs de `fiscal-year-close-button` et `fiscal-year-close-confirmation-action` diffèrent de
  celles de toutes les clés `*-close` et `*-dismiss` de la locale ; et elles égalent les valeurs du
  tableau. Mutation : remettre `Schliessen` → rouge.

### AC 11 — CHANGELOG et feuille de route

- `CHANGELOG.md`, `## [0.13.0] — Non publié`, `### Corrigé` : une entrée « **Manuels et libellés : ce que
  Kesh fait réellement** » qui énumère, avec le lien de chaque issue — taux de TVA du manuel (#539) ;
  menu « Paramètres » (#547) ; trois plans comptables, sans import de plan ni de contacts fictifs (#488,
  #291) ; import de factures déclenché à la main (#458) ; KF-036 corrigée depuis la v0.10.0 (#449) ;
  liens d'issues du README (#432) ; messages de réouverture qui disent l'ordre (#569) ; `MWST` (#321) ;
  clôture d'exercice distincte de la fermeture d'un panneau (#323).
- README « Feuille de route » : rien à changer (aucune fonctionnalité livrée ni retirée du tableau) ;
  vérifier seulement que la section « Fonctionnalités » ne promet plus de surveillance de dossier (AC 5).

## Tasks / Subtasks

- [ ] **T0 — Rebase et relevé** (AC tous)
  - [ ] `git fetch && git rebase origin/main` ; si 15-7b1, 15-13a ou 15-13b sont mergées, re-trouver chaque
        site **par la valeur** (commandes des AC) et noter les écarts au Dev Agent Record.
  - [ ] Recompter à la source les inventaires : 54 références README, 6 clés ×4 de #569, 6 `MwSt`.
- [ ] **T1 — Manuel utilisateur, brochure, manuel d'administration** (AC 1-6, 8)
  - [ ] AC 1 (2 sites), AC 2 (4 sites), AC 3 (user 204/308/314/409-422/426/437, admin 79/1361-1372,
        brochure 260/385), AC 4 (857-859), AC 5 (user 1524, brochure 396), AC 6 (admin 1931/1947),
        AC 8 (user 1456/1478/1773/2316-2317).
  - [ ] `scripts/mem-guard.sh make -C docs/manual fr` ; contrôle aplati de chaque texte neuf (présent) et
        ancien (absent) dans les trois PDF.
- [ ] **T2 — `api-external.md` et README** (AC 6, 7, 8, 5)
- [ ] **T3 — Catalogues** (AC 2, 8, 9, 10) — 4 locales ensemble, parité verte.
- [ ] **T4 — Replis** (AC 2, 8) — Rust (6) et frontend (5) égaux au fr-CH ; retirer les 3 commentaires `#569`.
- [ ] **T5 — Tests** : T1 (`vat_rates.rs`), T3/T8/T10/T11 (`kesh-i18n/src/loader.rs` `mod tests`),
      T2/T4/T5/T6/T7/T9 (nouveau `crates/kesh-api/tests/textes_coherents.rs`, sans base) ; Vitest des
      cinq fichiers de l'AC 8.
- [ ] **T6 — Mutations** : chacune de la liste ci-dessous observée **rouge**, puis restaurée (`git checkout`
      **puis `touch`** du fichier — mémoire *Mutation restaurée, binaire périmé*).
- [ ] **T7 — CHANGELOG** (AC 11).
- [ ] **T8 — Gates** : fmt + clippy workspace ; `scripts/test-fast.sh` (gate complet : la story touche
      `kesh-api`) ; frontend `npm run check`, `lint-i18n-ownership`, `test:unit`, `build` ; **E2E complet au
      dernier commit de code** (D7) — les libellés fr changent sur des écrans E2E (annulations refusées,
      soldes de départ), juger chaque rouge contre `docs/testing.md` § « Les échecs attendus ».

## Tests et mutations

| Test | Lieu | Mutation qui doit le faire rougir |
|---|---|---|
| T1 `le_manuel_cite_les_taux_poses_par_kesh` | `kesh-db/src/repositories/vat_rates.rs` (`mod tests`) | `3.7\%` réécrit au manuel ; `380` → `370` dans `DEFAULT_SWISS_RATES` |
| T2 `aucun_renvoi_au_menu_reglages` | `kesh-api/tests/textes_coherents.rs` | `\emph{Réglages}` remis à `user-manual.tex` |
| T3 `le_message_du_compte_de_produit_renvoie_a_l_ecran_reel` | `kesh-i18n/src/loader.rs` | « dans les Réglages » remis en fr-CH |
| T4 `plans_comptables_reels` | `textes_coherents.rs` | « Sterchi PME » remis ; `\emph{Import CSV}` remis (contacts) |
| T5 `aucun_dossier_surveille` | `textes_coherents.rs` | « dossier surveillé » remis à `user-manual.tex` |
| T6 `la_faille_kf036_n_est_pas_annoncee_ouverte` | `textes_coherents.rs` | `[Unreleased]` remis à `api-external.md` |
| T7 `references_d_issues_du_readme_sont_des_liens` | `textes_coherents.rs` | un `[#164]` nu ; un lien au mauvais numéro |
| T8 `les_prescriptions_de_reouverture_disent_l_ordre` | `loader.rs` | ancienne valeur de `error-fiscal-year-reopen-blocked` ; exemption d'une clé qui ne matche plus le verbe |
| T9 `les_replis_rust_de_reouverture_suivent_le_catalogue` | `textes_coherents.rs` | ancien repli d'`errors.rs:3085` |
| T10 `glossaire_mwst` | `loader.rs` | `reports-vat = MwSt` |
| T11 `la_cloture_d_exercice_ne_parle_pas_comme_un_panneau` | `loader.rs` | `fiscal-year-close-button = Schliessen` |
| Vitest (5 fichiers de l'AC 8) | frontend | ancien repli de `settlement-cancel-blocked.ts:39` |

`textes_coherents.rs` lit ses fichiers par `env!("CARGO_MANIFEST_DIR")` + `../../` ; chaque assertion
négative s'accompagne d'une assertion positive sur le même fichier (le fichier a été lu, il contient la
section attendue) : un chemin faux ne doit pas rendre un vert muet (mémoire *Tests qui prouvent moins*).

## Dev Notes

### Modules touchés, et dérogation à la règle de découpage

Paquets de code : **kesh-i18n** (catalogues, tests), **kesh-api** (`errors.rs`, `routes/fiscal_years.rs`,
`routes/opening_balances.rs`, un fichier de test), **frontend** (quatre replis, un repli d'`InvoiceForm`,
cinq tests), plus un test de module dans **kesh-db** (`vat_rates.rs`, sans code de production). Comptés en
modules métier de premier niveau, les replis de #569 en touchent plus de cinq (exercices, soldes de départ,
règlements, rapprochement, factures fournisseurs, facturation). **Dérogation assumée** : chaque changement
est un littéral de texte aligné sur un catalogue, sans logique ; la règle vise l'étendue d'un modèle
mental adversarial, qu'un remplacement de chaînes ne sollicite pas. Le découpage réel de la 15-14 est fait
ailleurs, sur une dépendance (C-15-14-1).

### Pièges

- **Apostrophes** : chaque clé garde son style (droite `'` ou typographique `’`) ; le repli Rust/TS est
  **octet pour octet** le catalogue — une apostrophe différente rend un vert du test frontend et un faux
  message côté API.
- **Placeholders Fluent** : `{ $name }` dans `opening-balances-locked-first-year-closed` ; le repli Svelte
  `:382` porte la même forme.
- **PDF** : régénérer **les trois** (`make fr`), sous `mem-guard` (un `lualatex`/`xelatex` emballé a déjà
  emporté une session, § *Plafonds mémoire*). Vérifier que `\keshVersion` n'est pas touché (pas de release).
- **Garde LIFO, rappel** : la réouverture va du plus récent vers l'ancien (`fiscal_years.rs:228`) ;
  « jusqu'à celui-ci » est vrai parce qu'on ne peut rouvrir un exercice que si aucun plus récent n'est clos.
- **`README.md:211`** reçoit des liens (AC 7) mais pas de réécriture de texte (AC 5).

### Références

- Issues : #539, #547, #488, #291, #458, #449, #432, #569, #321, #323 ; #459 ; #579, #324 (écartées).
- Registre : C-15-14-1 à C-15-14-8 ; C111, C122 (15-12a) ; C-15-12b-4 (B-1 = E-1).
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

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-10-09 — Spécification (Opus 5.5), sur `dc4bc58b`. Choix C-15-14-1 à C-15-14-8.

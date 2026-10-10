# Story 26-3 : Le manuel décrit les écrans réels, et la garde de `kesh-restore.sh` échoue fermée

Status: review

<!-- Spécifiée le 2026-10-10 sur fe162504 (branche story/26-3, worktree kesh-26-3), en autonomie
     (consignes communes des agents, memory/consignes-agents-epic15.md). Choix C-26-3-1 à 8 au registre
     _bmad-output/implementation-artifacts/epic-26-choix-autonomes.md. Numéros de ligne : ceux de
     fe162504 ; les 26-1 et 26-2, développées en parallèle, touchent les mêmes fichiers (voir
     « Conflits attendus ») — le T0 relève les sites de nouveau, par les commandes, jamais de confiance.
     Validation P1 (Sonnet, lentilles R et F, sur 33002beb) remédiée : choix C-26-3-9 à 12 ; bilan au
     Change Log. Validation P2 (Opus, R et F, sur 15422ee4) remédiée : signal D5 (recyclage), AC 6
     refondue par simplification (faux docker) ; choix C-26-3-13 à 17. Validation P3 (Sonnet, R et F,
     sur 46868649) remédiée : faux docker précisé, cas S1-S23, G14 lit up ; choix C-26-3-18 à 20. -->

## Story

En tant qu'utilisateur qui prépare son premier import de relevé, et qu'exploitant d'un NAS Synology
dont le compose n'est pas exactement celui qui est livré,
je veux que le manuel me décrive les écrans et les formats tels qu'ils sont, et que le script de
restauration refuse de recharger tant qu'il ne sait pas que Kesh est arrêté,
afin de ne pas chercher une case qui n'existe pas, ni un export que ma banque n'offre pas, et que le
script refuse de commencer un rechargement tant qu'un Kesh tourne (la garde est lue une fois, avant
l'étape 2 : un Kesh démarré pendant le rechargement n'est pas vu — validation P1, LOW-5).

| Issue | Prio. | Labels | Objet | État au code (`fe162504`) |
|---|---|---|---|---|
| [#616] | P3 | bug, documentation | case « Compte de banque » du plan comptable, inexistante | **non corrigé** : `user-manual.tex:345`, `:1652` |
| [#617] | P3 | bug, documentation | CAMT.053 « fourni par toutes les banques suisses » | **non corrigé** : `user-manual.tex:1638` ; zone de dépôt de l'écran : « fichier CAMT.053 » dans les 4 catalogues |
| [#619] | P2 | bug | garde de `kesh-restore.sh` muette si le service est renommé ; réseau du dump non documenté | **non corrigé** : `kesh-restore.sh:76` ; `admin-manual.tex:1671` |

Les trois portent `bug` : elles comptent dans l'engagement 1 (§ *Priorités des défauts*). PR prévue :
`closes #616`, `closes #617`, `closes #619` **dans le titre ou le corps de la PR** (merge en squash) ;
commits intermédiaires en `refs #616 #617 #619`.

[#616]: https://github.com/guycorbaz/kesh/issues/616
[#617]: https://github.com/guycorbaz/kesh/issues/617
[#619]: https://github.com/guycorbaz/kesh/issues/619

## Acceptance Criteria

> **Contrôle des PDF** (AC 10) : fonction `occ` de la Story 15-14a, à coller telle quelle, sous
> `LC_ALL=C.UTF-8` — un `grep -F` sur `pdftotext` brut rend des faux négatifs (apostrophes
> typographiques, traits d'union coupés, ligatures) :
>
> ```sh
> export LC_ALL=C.UTF-8
> norm() { sed "s/[’‘ʼ]/'/g; s/ﬁ/fi/g; s/ﬂ/fl/g; s/ﬀ/ff/g; s/ﬃ/ffi/g; s/ﬄ/ffl/g; s/\xc2\xad//g; s/\xc2\xa0/ /g; s/\xe2\x80\xaf/ /g; s/\xe2\x80\x89/ /g" | tr -d '\n -'; }
> occ() { pdftotext "$1" - | norm > "${TMPDIR:-/tmp}/aplati"; printf '%s' "$2" | norm > "${TMPDIR:-/tmp}/motif"; grep -oF -f "${TMPDIR:-/tmp}/motif" "${TMPDIR:-/tmp}/aplati" | wc -l; }
> ```
>
> **Chaque contrôle « ancien absent » est précédé du même `occ` sur le PDF d'avant**
> (`git show fe162504:docs/manual/fr/<f>.pdf > "$TMPDIR/avant.pdf"`), qui doit rendre ≥ 1. Relevé à la
> spécification sur les PDF de `fe162504` : `Compte de banque` → **2**, `marqué « Compte de banque »` →
> **1**, `fourni par toutes les banques suisses` → **1**, `Nouvel import CSV` → **1** (user) ;
> `quel que soit le nom du projet` → **1**, `SAUVEGARDE_SERVICE` → **0** (admin). La liste complète des
> contrôles est à l'AC 10.2 ; `SAUVEGARDE_RESEAU (frontend` n'y est **pas** un « ancien absent » : G16 (g)
> l'exige au manuel (validation P1 LOW-1, P2 F-L4). Et pour le contrôle visuel, la forme aplatie demandée par le
> `CLAUDE.md` : `pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`.

### AC 1 — Le plan comptable : la liste des champs est celle du formulaire (#616, C-26-3-1)

**Inventaire clos** — les champs des deux dialogues de `frontend/src/routes/(app)/accounts/+page.svelte`
(relever par `grep -n '<label' frontend/src/routes/\(app\)/accounts/+page.svelte`) :

- création (`:532`-`:590`) : **Numéro**, **Nom**, **Type** (Actif, Passif, Produit, Charge —
  `ACCOUNT_TYPES`, `AccountType` à quatre variantes dans `kesh-core`), **Compte parent (optionnel)**,
  **Rôle**, **Postable** ; bouton « Nouveau compte » (`accounts-add`) ;
- édition (`:618`-`:660`) : Numéro (**toujours** désactivé — « Le numéro n'est pas modifiable après
  création. », `accounts-edit-description`), Nom, Type, Rôle, Postable — **pas** de compte parent.

Exigences :

1. `user-manual.tex` § *Ajout d'un compte* (`:333`-`:346`) ne décrit **que** ces champs, sous leurs
   libellés d'écran (valeurs `fr-CH` des catalogues, pas les replis Svelte). Disparaissent :
   « Compte de banque », « Devise », « Libellé » (→ « Nom »), les types « Bilan / Résultat ».
2. La liste est suivie d'une phrase qui dit **comment un compte devient compte bancaire** : on ne le
   marque pas dans le plan comptable ; on déclare le compte bancaire dans *Administration → Comptes
   bancaires* (*Nouveau compte bancaire* : Banque, IBAN, QR-IBAN (optionnel), Compte principal — libellés d'écran,
   `messages.ftl:1229`-`:1259` ; si la 26-2 est mergée avant, renvoyer à sa § *Comptes bancaires* au lieu
   de relister les champs) et on le lie par
   **« Compte comptable lié »** (ou, pour un compte bancaire existant, le bouton **« Lier au plan
   comptable »**). Seuls y sont proposés les comptes **actifs, postables, de type Actif ou Passif**,
   hors compte débiteurs et compte créanciers désignés (`bank-accounts/+page.svelte:105`-`:115`). Renvoi
   vers ce que le manuel dit déjà de ce lien (`user-manual.tex:240`-`:272`, solde comptable).
3. § *Modification d'un compte* (`:348`-`:350`) : **inventaire clos de ses affirmations** (validation
   P1, R-6), chacune corrigée contre l'écran :
   - « Cliquer sur un compte ouvre son détail » — **faux** : aucune ligne n'est cliquable ; on modifie
     par le bouton crayon de la ligne (`openEdit`, `+page.svelte:496`, `aria-label` « Modifier le
     compte N ») ;
   - « modifier le **libellé**, le **compte parent**, le rôle et l'indicateur postable » — le champ
     s'appelle **Nom** ; le compte parent **n'est pas modifiable** (ni à l'écran ni par l'API :
     `UpdateAccountRequest` n'a pas de `parent_id`, `routes/accounts.rs:71`-`:86`, vérifié en P1) ;
   - « Le numéro n'est pas modifiable une fois des écritures saisies » — **faux** : il n'est jamais
     modifiable après création (`disabled` sans condition, `:619`).
   Le reste du paragraphe (type modifiable, avertissement de reclassement) est juste et reste.
4. Les exemples de comptes cités restent conformes aux gardes G4-bis et G4-ter
   (`crates/kesh-api/tests/textes_coherents.rs` : tout numéro cité existe dans un plan livré, sous son
   nom).

### AC 2 — L'import : le compte bancaire cible et la procédure réelle (#616, C-26-3-1)

L'écran réel (`/bank-import`, menu *Quotidien → Importer des relevés*, `nav-bank-import`) :
sélecteur **« Compte bancaire cible »** (comptes bancaires non archivés de la société, venus de
`GET /api/v1/companies/current`) ; sélecteur de profil CSV s'il existe au moins un profil ; zone de
dépôt ; **prévisualisation** avec ses avertissements à confirmer ; bouton **« Confirmer l'import »**.

1. `user-manual.tex` § *Import dans Kesh* (`:1648`-`:1655`) et § *Import avec un profil existant*
   (`:1683`-`:1690`) décrivent ces étapes, sous ces libellés — plus de « Quotidien → Importer →
   Nouvel import », ni de « Nouvel import CSV », ni de bouton « Importer » qui n'existe pas.
2. L'étape du compte cible dit : le compte bancaire est déclaré dans *Administration → Comptes
   bancaires* (renvoi à l'AC 1, point 2) ; **l'import n'exige pas le lien au plan comptable**, la
   **réconciliation** l'exige (un compte bancaire sans compte lié y est refusé,
   `BANK_ACCOUNT_NOT_CONFIGURED`). Vérifier les deux affirmations au code avant de les écrire.
3. Le profil CSV (validation P1 R-13, **réécrit en P2, F-M6**, vérifié au code à la remédiation P2) :
   l'aperçu est calculé **au dépôt du fichier** avec le profil alors choisi (`previewBankImport(file, …,
   { bankProfileId })`, `BankImportUpload.svelte:106`-`:108`) ; s'il n'y en a pas, l'écran présélectionne
   le profil reconnu **après** l'aperçu (`:110`-`:116`). Changer de profil **après** l'aperçu ne le
   recalcule pas (`onChange={(id) => (bankProfileId = id)}`, `:222`), et « Confirmer l'import » envoie le
   **nouveau** profil (`buildFlags()`, `:73`-`:82`) avec l'aperçu de l'ancien. La procédure écrite est donc :
   **choisir le profil avant de déposer le fichier** quand on le connaît ; sinon, après l'aperçu, si le
   profil proposé n'est pas le bon : **Annuler** (`bank-import-cancel`, `:592`), choisir le profil,
   **redéposer** le fichier — `reset()` (`:61`-`:71`) ne remet pas `bankProfileId` à zéro, le choix tient.
   Le manuel ne décrit **jamais** « changer le profil puis confirmer ». Il dit aussi qu'après **Annuler**,
   le profil — choisi ou présélectionné — **reste** choisi pour le fichier suivant (relevé P3). Le défaut d'écran lui-même
   (aperçu non recalculé) n'est **pas** corrigé ici : issue P2 à ouvrir par l'orchestrateur (C-26-3-15).
3-bis. Le rapport d'import (`user-manual.tex:1657`-`:1663`, « Nombre de doublons détectés (ignorés) ») est
   vérifié contre l'écran : les doublons sont ignorés **par défaut**, importés si l'utilisateur choisit
   « Importer quand même » (`bank-import-labels-confirm-duplicate-lines-import`,
   `BankImportUpload.svelte:390`-`:402`) ; le texte le dit (validation P2, R-L12).
4. **Renvois au chemin de l'écran, inventaire clos** (validation P1, R-8 = MEDIUM-3) — relevé par
   `git grep -nIE "emph\{Importer\}|emph\{Import bancaire\}|Quotidien → Importer\*\*|Nouvel import" -- docs`
   (**7** lignes sur `fe162504` : `user-manual.tex:1651`, `:1654`, `:1686`, `:1690` — couvertes par
   l'AC 2.1 —, `:1756`, `:1826`, `getting-started.md:74` ; **0 attendu** après correction) et
   `git grep -nI "Import bancaire CAMT.053" -- docs` (valeur de la clé que l'AC 4 change) :
   | Site | Aujourd'hui | Cible |
   |---|---|---|
   | `user-manual.tex:1756` | détail de son import (*Import bancaire*, puis l'import) | *Quotidien → Importer des relevés*, liste *Imports précédents*, nom du fichier (`BankImportList.svelte:34`, `/bank-import/{id}`) |
   | `user-manual.tex:1826` | détail de l'import bancaire (*Import bancaire*, puis l'import concerné) | idem |
   | `docs/user-guide/fr/getting-started.md:74` | Menu **Quotidien → Importer** | **Quotidien → Importer des relevés** |
   | `docs/i18n-glossaire.md:151` | colonne **« précédent »** : `Import bancaire CAMT.053` (valeur de `bank-import-labels-page-title`, que l'AC 4 change) | **`Import bancaire` (`audit-log-entity-bank-imports`)** — valeur et clé, dans le style des lignes voisines (validation P3, R-L3) ; la clé atteste encore les quatre formes : `Import bancaire` / `Bankimport` / `Importazione bancaria` / `Bank import` (`messages.ftl` fr `:2266`, de `:2212`, it `:2213`, en `:2218`) ; les quatre termes de la ligne ne changent pas (validation P2, F-M8, C-26-3-16) |
   Hors inventaire, justes : `\section{Import bancaire}` (`:1628`, titre de chapitre du manuel, non un
   chemin d'écran) ; `admin-manual.tex:81`, `:774`, brochure `:162`, `:262`, `:388`, README `:41`
   (nom de fonctionnalité) ; `audit-log-entity-bank-imports = Import bancaire` (type d'entité du
   journal d'audit, non l'écran) ; `docs/kesh-specifications.txt` (spécification historique, non
   corrigée) ; commentaires de story `errors.rs:644`, `:2032`, `routes/bank_imports.rs:1`,
   `messages.ftl:1043` (« Story 8-1b — Import bancaire CAMT.053 », généalogie).
5. **« Récupération du fichier CAMT.053 »** (`:1640`-`:1646`, C-26-3-12) : trois affirmations sur des
   e-banking tiers que rien ne vérifie (libellés « Téléchargement / Extraits », « Multibanking ou
   Klippa », chemin *PostFinance Yellownet*), de la même famille que #617. Réécrire en consigne
   générale non nominative (chercher dans l'e-banking l'export des extraits au format CAMT.053 /
   ISO 20022 XML ; le libellé varie selon la banque ; à défaut, CSV — renvoi AC 3) ; supprimer les
   chemins de menus tiers.
6. **Renvoi à la section des comptes bancaires** : si la 26-2 est mergée avant (elle crée
   `\section{Comptes bancaires}\label{sec:comptes-bancaires}`), les renvois des AC 1.2 et 2.2 portent
   `\S\ref{sec:comptes-bancaires}` ; sinon, le texte seul, et le signaler dans le rapport final
   (validation P1, LOW-6).
7. **Hors périmètre** : § *Profils banque* (`:1669`-`:1681`, création d'un profil). La Story 26-1 y
   ajoute ses réglages et la réécrit ; ne pas la toucher ici (conflit certain). Si la 26-1 est mergée
   avant le développement, vérifier que ses étapes sont justes et le consigner au Dev Agent Record ;
   sinon, le signaler à l'orchestrateur dans le rapport final.

### AC 3 — Le CAMT.053 n'est pas universel ; la voie CSV est indiquée (#617, C-26-3-2)

1. `user-manual.tex:1638` dit que **la plupart** des banques suisses proposent le CAMT.053 (ISO 20022),
   **pas toutes** ; qu'à défaut — certaines offres d'e-banking ne proposent que le CSV ou le MT940 —
   on importe le **CSV avec un profil banque** (renvoi `\S\ref{sec:import-csv}` : poser ce `\label`
   sur `\subsection{Import CSV multi-encodage}`, `:1665`, ou réutiliser celui de la 26-1 s'il existe
   déjà) ; que le CSV ne porte pas le solde de clôture, si bien que **le contrôle du solde se fait à la
   main** après l'import (rapprocher le solde comptable du compte du solde de l'extrait) ; que **le
   MT940 n'est pas lu** par Kesh (demande ouverte [#618]).
2. Grep du symptôme sur tout le dépôt suivi, **relevé à la spécification** — à rejouer au T0 et au
   dernier commit :
   ```sh
   git grep -nIiE "toutes les banques|all (swiss )?banks|alle (schweizer )?banken|tutte le banche|banques? suisses?|swiss banks?" -- docs website README.md frontend/src crates/kesh-i18n
   ```
   Sur `fe162504` : **4** lignes (recompté en remédiation P2, R-L7 = F-L5) — `user-manual.tex:1638` (le
   site), `user-manual.tex:817` (« exigé par les banques suisses sur les QR-factures », juste),
   `website/about.html:116` (« format expected by Swiss banks », pain.001, juste) et
   `docs/kesh-prd-v0.2.md:252` (« scannable par une banque suisse », juste). Brochure, README, site : aucune promesse d'universalité
   (`git grep -niI camt -- website README.md docs/manual/fr/marketing-brochure.tex` relu ligne à ligne).

[#618]: https://github.com/guycorbaz/kesh/issues/618

### AC 4 — L'écran d'import ne dit plus « CAMT.053 » seul (#617, C-26-3-2, C-26-3-17)

Sites, **inventaire clos** relevé par `git grep -n "bank-import-labels-drop-zone\|bank-import-labels-page-title" -- frontend/src crates/kesh-i18n` et `grep -n 'aria-label="Glisser' frontend/src/lib/features/bank-import/BankImportUpload.svelte` :

| Site | Aujourd'hui | Cible (`fr-CH`) |
|---|---|---|
| `bank-import-labels-drop-zone`, 4 catalogues (`fr-CH:1077`, `de-CH`/`en-CH`/`it-CH:1018`) | « Glissez votre fichier CAMT.053 ici… » | « Glissez votre fichier CAMT.053 ou CSV ici ou cliquez pour parcourir » (= repli actuel, `BankImportUpload.svelte:258`) |
| `aria-label` en dur, `BankImportUpload.svelte:232` | « …un fichier CAMT.053 ou cliquer… » | **lu par `i18nMsg('bank-import-labels-drop-zone', …)`** (même texte que la zone, traduit, tenu par G20 — validation P2, F-L7) |
| `bank-import-labels-page-title`, 4 catalogues (`fr-CH:1075`, autres `:1016`) + repli `bank-import/+page.svelte:52` | « Import bancaire CAMT.053 » | « Import de relevés bancaires » |
| `<title>` en dur, `bank-import/+page.svelte:48` | « Import bancaire - Kesh » | **`{i18nMsg('bank-import-labels-page-title', …)} — Kesh`** (tiret long) — clé existante ; la forme lue par clé et le tiret long sont ceux de la majorité des titres i18n (`} — Kesh` : 16 occurrences **dans un `<title>`** sous `frontend/src/routes` — 17 lignes au grep brut, la 17e, `reconciliation/rules/+page.svelte:65`, n'étant pas un `<title>` (P4 ciblée, L3) —, contre 8 `} - Kesh`, relevé P3 ; précédents `credit-notes/+page.svelte:30`, `open-items/+page.svelte:16`) — validation P3, F-12 |
| `docs/i18n-glossaire.md:151`, colonne « précédent » | `Import bancaire CAMT.053` | `Import bancaire` (`audit-log-entity-bank-imports`) (AC 2.4) |

**Cibles des quatre locales, fixées** (validation P2, R-L6 = F-L6 ; terme attesté `relevé (bancaire)` =
`Kontoauszug` / `estratto conto` / `statement`, `i18n-glossaire.md:105`, figé par `nav-bank-import`) :

| Clé | fr-CH | de-CH | en-CH | it-CH |
|---|---|---|---|---|
| `bank-import-labels-page-title` | Import de relevés bancaires | Import von Kontoauszügen | Bank statement import | Importazione di estratti conto |
| `bank-import-labels-drop-zone` | Glissez votre fichier CAMT.053 ou CSV ici ou cliquez pour parcourir | CAMT.053- oder CSV-Datei hier ablegen oder klicken zum Durchsuchen | Drop your CAMT.053 or CSV file here or click to browse | Trascinare il file CAMT.053 o CSV qui o fare clic per sfogliare |

**Couverture** (validation P1 R-10, P2 F-L7) : les deux clés sont tenues par G20 (AC 9) dans les quatre
catalogues ; l'`aria-label` et le `<title>` les lisent désormais, si bien qu'aucun texte de l'écran d'import
ne reste en dur. Les replis Svelte valent la valeur `fr-CH` — **non gardé par un test** :
`i18n-un-repli-par-cle.test.ts` ne couvre que d'autres préfixes (`:51`-`:60` ; validation P2, R-L9), à
vérifier à la main. Aucun test ne dépend de ces textes (E2E par `data-testid`,
`bank-import-page-title`, relevé : `grep -rn "Import bancaire\|Glissez" frontend/tests frontend/src --include=*.ts` → 0).

### AC 5 — La garde de `kesh-restore.sh` échoue fermée (#619, C-26-3-3)

Dans `scripts/synology/kesh-restore.sh`, étape 1 (`:75`-`:79`), **toujours avant le verrou** :

1. Nouveau réglage `SAUVEGARDE_SERVICE=${SAUVEGARDE_SERVICE:-kesh-api}`, déclaré avec les autres
   (`:60`-`:63`).
2. Une seule lecture de l'hôte, en **affectation nue** (une erreur de `docker` vaut refus, `set -e`),
   **séparateur non blanc imposé** (validation P1, R-5 = MEDIUM-1, C-26-3-9) :
   ```sh
   LISTE=$(docker ps --all --format '{{.State}}|{{.Image}}|{{.Label "com.docker.compose.service"}}|{{.Names}}')
   ```
   lue par `while IFS='|' read -r ETAT IMAGE SERVICE NOM RESTE; do … done <<< "$LISTE"` — boucle
   dans le shell courant (un `| while` mettrait `exit 1` dans un sous-shell : garde ouverte). Raisons :
   une étiquette de service **vide** (conteneur lancé hors compose, `docker run`) entre deux
   séparateurs blancs serait avalée par `read` et décalerait les champs ; avec `|`, elle reste un
   champ vide. Règles de lecture, toutes **fermées** :
   - ligne **vide** → ignorée (`<<< ""` d'un hôte sans conteneur fournit une ligne vide : elle ne doit
     ni compter ni refuser) ;
   - ligne non vide dont `ETAT`, `IMAGE` ou `NOM` est vide, ou qui porte un cinquième champ
     (`RESTE` non vide) → **refus** « ligne de docker ps illisible », `exit 1` ;
   - la boucle n'emploie ni `$1` ni `set --` ni `awk '{print $1}'` : G16 (c) exige **un seul** usage
     de `$1` dans le script (`\$\{?1\b`, validation P1, R-14).
3. Sont **gardés** : les conteneurs dont le service vaut `$SAUVEGARDE_SERVICE`, **et** ceux dont
   l'image est `gcorbaz/kesh` — `^(docker\.io/)?gcorbaz/kesh([:@].*)?$` —, quel que soit leur service.
   Tout conteneur gardé dans un autre état que `exited|created|dead` → refus, `exit 1`. La forme
   `case "$ETAT" in exited|created|dead) ;; *) … exit 1;; esac` est conservée (liste fermée,
   C-15-14-81). **Deux libellés distincts**, que la recette grepe (validation P1, R-3) :
   - conteneur du service : `kesh-restore : le conteneur <NOM> (service « <SERVICE> ») est « <ETAT> » — arrêtez-le d'abord ; rien n'a été touché`
     (le fragment `est « <ETAT> »` reste **littéral** : la section 4 de la recette le grepe, `recette.sh:159`) ;
   - filet, conteneur d'un autre service : `kesh-restore : le conteneur <NOM> (image Kesh <IMAGE>, service « <SERVICE> ») est « <ETAT> » — …`
     (le fragment `image Kesh` n'apparaît que dans ce message).
   Angles morts du filet, écrits dans l'en-tête, dans G16 et au manuel : **image affichée par son
   identifiant** — cas courant : après un `docker compose pull`, l'étiquette est **déplacée** vers la
   nouvelle image et le conteneur non recréé n'affiche plus que l'identifiant (relevé au poste,
   `407fccb51e71` ; validation P2, F-L1) —, image référencée par `index.docker.io/…` ou un miroir privé,
   image construite localement sous un autre nom (validation P1, LOW-5). Le cas S15 de la recette le fixe.
   Compteur des conteneurs du service : `N=$((N+1))`, **jamais** `((N++))`, qui rend 1 quand `N` vaut 0
   et tue le script sans message sous `set -e` (validation P2, R-L11 = F-L2).
4. **Aucun** conteneur du service `$SAUVEGARDE_SERVICE` sur l'hôte → refus, `exit 1`, libellé
   **sur une seule ligne de code** (G16 (c) refuse une ligne qui commence par `docker compose`,
   validation P1, LOW-4) :
   `kesh-restore : aucun conteneur du service « <SERVICE> » sur cet hôte (réglage SAUVEGARDE_SERVICE) — si le service de Kesh porte un autre nom dans votre compose, posez SAUVEGARDE_SERVICE=<nom> ; si son conteneur a été supprimé (docker compose down) ou n'a jamais été créé, créez-le sans le démarrer : docker compose up --no-start <service> ; rien n'a été touché`.
   **Une seule commande**, `docker compose up --no-start <service>`, dans le script **et** au manuel
   (validation P1, MEDIUM-4, C-26-3-11) : elle existe dans toute version de Compose v2, là où
   `docker compose create` est moins répandue. Angle mort écrit : elle interpole le compose entier —
   `.env` incomplet, elle échoue (le manuel le dit).
5. L'en-tête du script (`:5`-`:10`, `:20`, `:49`-`:51`) et le message final (`:127`, « docker compose
   start kesh-api ») disent la garde telle qu'elle est et emploient `$SAUVEGARDE_SERVICE`.
6. `kesh-dump.sh` n'a pas de garde et ne lit pas `SAUVEGARDE_SERVICE` ; il n'est pas modifié, sauf son
   en-tête si une phrase y décrit la garde (vérifier).

### AC 6 — La garde se prouve contre un faux `docker` (#619, C-26-3-13, C-26-3-14, C-26-3-18)

> **Refondue en remédiation P2** (signal D5, recyclage P1 MEDIUM-5 → P2 M1/M5 : la garde lit l'état
> Docker de **tout l'hôte**, que la recette ne maîtrise pas). Décision de l'orchestrateur, **par
> simplification, sans découpage** (C-26-3-13) : la nouvelle section de la recette **n'emploie aucun
> vrai conteneur**. Plus de pré-vol de l'hôte, d'étiquette d'image, de `compose_factice` paramétré,
> d'`up --no-start`, de conteneur hors compose ni de nettoyage pour ces cas ; plus aucune dépendance aux
> conteneurs `kesh-dev`/`kesh-api` du poste. **Précisée en remédiation P3** (C-26-3-18) : définition et
> portée du faux, environnement par cas, sélecteur de section, six cas neufs, cas renommés `S1`-`S23`
> (les noms `G14`-`G17` sont ceux de gardes existantes).

`scripts/synology/recette.sh` gagne une section **« 0. Garde de kesh-restore.sh contre un faux
docker »**, placée **avant** la mise en place (`recette.sh:92`) : elle ne lit ni ne crée aucun
conteneur. Elle compte ses échecs dans `ECHECS` par `ok`/`ko` (`recette.sh:38`-`:39`) **sans
interrompre** la recette.

**Sélecteur** (validation P3, R-L4 = F-7 ; **réécrit en P4 ciblée, M1**) : `RECETTE_SECTION=0 bash
scripts/synology/recette.sh` joue la section 0 **seule** et sort aussitôt après elle sur « RECETTE VERTE »
ou « RECETTE ROUGE (n échec(s)) ». **Ordre imposé dans `recette.sh`** — aujourd'hui `mktemp` (`:25`),
`openssl` (`:26`, `:28`) et surtout `trap nettoyer EXIT` (`:37`), dont la fonction appelle le **vrai**
`docker` (`docker rm -f`, `docker network rm`, `docker run … mariadb:10.11`, image tirée si absente,
erreurs vers `/dev/null`), s'exécutent avant toute section :
1. réglages sans effet (`RACINE`, `SCRIPTS`, noms), fonctions `ok`/`ko`/`verifier`, puis
   `W=$(mktemp -d …)` et une trap **minimale** `trap 'rm -rf "$W"' EXIT`, qui n'appelle pas `docker` ;
2. la section 0 ; en `RECETTE_SECTION=0`, verdict et `exit` **ici** ;
3. ensuite seulement : `openssl` (mots de passe), garde `kesh-mariadb-dev` (`:31`), `nettoyer` et
   `trap nettoyer EXIT` (qui remplace la trap minimale et efface `$W` comme aujourd'hui), puis la mise en
   place et les sections 1 à 9, inchangées.
**Prérequis exacts de ce mode** : `bash` ; coreutils (`mktemp`, `rm`, `cat`, `printf` interne, `dirname`,
`sha256sum`, `wc`) ; `gzip` ; `grep` ; `awk` (étape 3 de `kesh-restore.sh`). **Ni `docker`, ni `openssl`,
ni `sqlx`, ni `python3`, ni réseau** — et c'est vérifiable : en mode 0, aucun appel au vrai `docker` n'est
possible, puisque la seule trap posée est la trap minimale. Sans la variable, la recette entière tourne
comme aujourd'hui, section 0 en tête. C'est ce mode qui sert aux preuves de mutation.

**Le faux `docker` est une fonction bash, pas un exécutable du PATH** (C-26-3-14) : `kesh-restore.sh:54`
fait `export PATH="/usr/local/bin:/usr/bin:/bin:${PATH:-}"`, si bien qu'un faux placé en tête du `PATH`
de l'appelant serait **masqué par `/usr/bin/docker`**. Une fonction exportée est importée par le `bash`
qui exécute le script et **précède toute recherche dans le `PATH`**.

**Portée de la fonction, écrite** (validation P3, R-M2 = F-5, F-6, C-26-3-18) :
- la fonction est **définie ET exportée à l'intérieur du sous-shell de chaque cas** — par exemple une
  fonction `faux_docker_installer` qui contient la définition `docker() { … }` puis `export -f docker`, et
  qui n'est **appelée que** dans `( … )` ; **jamais** de définition de `docker` au niveau du script : elle
  masquerait le vrai `docker` pour la mise en place (`recette.sh:93`), `sql()` (`:41`), `compose_factice`
  et la `trap nettoyer EXIT` (`:37`), dont les erreurs partent à `/dev/null` (`:33`-`:35`) — conteneurs de
  recette laissés sur l'hôte **sans message** ;
- **assertion de montage** en tête de la section 1 (et, en mode complet, juste après la section 0) :
  `verifier "le vrai docker répond après la section 0" "$(type -t docker)" file` ;
- les variables du faux, `FAUX_SORTIE`, `FAUX_JOURNAL`, `FAUX_CODE`, sont **exportées dans le même
  sous-shell** (le `bash` enfant ne voit que l'environnement ; sous `set -u`, `kesh-restore.sh:52`, une
  variable absente tuerait la fonction importée sur « unbound variable ») ; `FAUX_SORTIE` vaut un
  **fichier** par cas, **toujours créé**, vide pour S5 ; `FAUX_CODE` vaut `0` sauf pour S17 ;
- **environnement isolé par cas** (R-L4) : le sous-shell commence par
  `unset SAUVEGARDE_DOSSIER SAUVEGARDE_BASE SAUVEGARDE_RESEAU SAUVEGARDE_IMAGE SAUVEGARDE_SERVICE`, puis
  pose ceux du cas — un shell d'exploitant qui exporte `SAUVEGARDE_SERVICE` ne fausse aucun cas ;
- la sortie du script est capturée **avec** la sortie d'erreur (`2>&1`) : les motifs sont des messages
  d'erreur ;
- dans la fonction, les motifs de `case` qui contiennent `|` sont **entre guillemets** (un `|` nu est une
  alternative de `case`).

Comportement de la fonction :
- elle **journalise** chaque appel (`printf '%s\n' "$*" >> "$FAUX_JOURNAL"`) ;
- pour les deux formes de `ps` ci-dessous, si `FAUX_CODE` ≠ 0 : écrit « Cannot connect to the Docker
  daemon » sur la sortie d'erreur et rend `FAUX_CODE` ;
- `ps --all --format {{.State}}|{{.Image}}|{{.Label "com.docker.compose.service"}}|{{.Names}}` (forme
  exacte de l'AC 5.2) : rend le contenu de `FAUX_SORTIE` tel quel ;
- `ps --all --filter label=com.docker.compose.service=<S> --format {{.State}}` (forme **d'origine**,
  `fe162504:kesh-restore.sh:76`, pour la seule preuve de mutation (O)) : rend la **première colonne** des
  lignes de `FAUX_SORTIE` dont la **troisième** vaut `<S>` (découpage `|`) ;
- **tout autre appel** — `run`, `exec`, `compose`, `rm`, une autre forme de `ps` — : écrit
  `FAUX DOCKER : appel inattendu : <args>` sur la sortie d'erreur **et** `INATTENDU <args>` au journal,
  rend **97**. Le rechargement, la sonde et le dump de sécurité passant tous par `docker run`, **aucune
  action sur une base n'est possible** sous le faux : c'est la preuve que le script s'arrête avant.

**Montage commun**, sous `$W/garde/` (aucun fichier de `$W/kesh`) : un dossier `SAUVEGARDE_DOSSIER`
portant un `kesh-restore.cnf` factice (contenu indifférent : le script n'en teste que la présence,
`kesh-restore.sh:65`) ; un dossier source portant un dump **fabriqué** — `printf 'CREATE DATABASE
\x60kesh\x60;\n' | gzip > kesh_pre_backup.sql.gz` et son `.sha256` —, sans base ni conteneur. Chaque cas
lance, dans son sous-shell, `SAUVEGARDE_DOSSIER=$W/garde/dossier SAUVEGARDE_BASE=autre
[SAUVEGARDE_SERVICE=…] bash "$SCRIPTS/kesh-restore.sh" "$W/garde/source" 2>&1`. **`SAUVEGARDE_BASE=autre`
est gardé** : quand la garde passe, le script s'arrête de lui-même à l'étape 3 sur son propre message
(« le dump recrée la base « kesh », non « autre » », `kesh-restore.sh:93`), sans atteindre la sonde ;
l'oracle « la garde passe » est ainsi un message **du script**, non un refus du faux.

**Classes de verdict, et leur motif** (validation P2 R-L3 = F-M4 : on exige le motif de **sa** classe et
l'**absence des motifs des autres classes**) :

| Classe | Motif exigé (`grep -F`) | Motif cherché pour l'**absence** |
|---|---|---|
| ZÉRO | `aucun conteneur du service « <S> »` | `aucun conteneur du service` |
| SERVICE | `(service « <S> ») est « <ETAT> »` | expression `\(service « [^»]* »\) est « ` (`grep -E`) |
| FILET | `image Kesh` | `image Kesh` |
| ILLISIBLE | `ligne de docker ps illisible` | `ligne de docker ps illisible` |
| PASSE | `recrée la base` | `recrée la base` |
| ERREUR | aucun des cinq motifs ; code non nul | — |

Le message du filet porte `, service « X ») est` et jamais `(service « X ») est` : c'est la parenthèse
ouvrante qui sépare SERVICE de FILET (relevé P3).

**Les cas** (lignes de `FAUX_SORTIE` séparées par ` ⏎ `, écrites telles quelles par `printf` ; `<S>` =
`kesh-api` sauf réglage). Nommés **`S1`-`S23`** (validation P3, R-L1) :

| Cas | `FAUX_SORTIE` | Réglage | Classe |
|---|---|---|---|
| S1 service renommé, image locale | `running\|kesh-local:dev\|kesh\|kesh-kesh-1` | — | ZÉRO |
| S2 idem, réglé | idem | `SAUVEGARDE_SERVICE=kesh` | SERVICE (`kesh`, `running`) |
| S3 orphelin arrêté, renommé Kesh en marche | `exited\|gcorbaz/kesh:0.12.1\|kesh-api\|kesh-api` ⏎ `running\|gcorbaz/kesh:0.13.0\|kesh\|kesh-kesh-1` | — | FILET |
| S4 réglé, orphelin `kesh-api` en marche | `exited\|gcorbaz/kesh:0.13.0\|kesh\|kesh-kesh-1` ⏎ `running\|gcorbaz/kesh:0.12.1\|kesh-api\|kesh-api` | `SAUVEGARDE_SERVICE=kesh` | FILET |
| S5 aucun conteneur | *(fichier vide)* | — | ZÉRO, et `up --no-start` dans le message |
| S6 `created` | `created\|gcorbaz/kesh:0.13.0\|kesh-api\|kesh-api` | — | PASSE |
| S7 `restarting` | `restarting\|gcorbaz/kesh:0.13.0\|kesh-api\|kesh-api` | — | SERVICE |
| S8 `paused` | `paused\|gcorbaz/kesh:0.13.0\|kesh-api\|kesh-api` | — | SERVICE |
| S9 hors compose (étiquette vide) | `running\|mariadb:10.11\|\|kesh-recette-db` ⏎ `exited\|gcorbaz/kesh:0.13.0\|kesh-api\|kesh-api` | — | PASSE |
| S10 ligne vide intercalée | `exited\|gcorbaz/kesh:0.13.0\|kesh-api\|kesh-api` ⏎ *(vide)* ⏎ `running\|mariadb:10.11\|\|x` | — | PASSE |
| S11 ligne incomplète | `exited\|gcorbaz/kesh:0.13.0\|kesh-api\|kesh-api` ⏎ `running\|mariadb:10.11` | — | ILLISIBLE |
| S12 cinq champs | `exited\|gcorbaz/kesh:0.13.0\|kesh-api\|kesh-api` ⏎ `running\|mariadb:10.11\|a\|b\|c` | — | ILLISIBLE |
| S13 deux projets, même service | `exited\|gcorbaz/kesh:0.13.0\|kesh-api\|p1-kesh-api` ⏎ `running\|gcorbaz/kesh:0.13.0\|kesh-api\|p2-kesh-api` | — | SERVICE |
| S14 image par empreinte, autre service | `running\|docker.io/gcorbaz/kesh@sha256:<64 hex>\|autre\|x` ⏎ `exited\|gcorbaz/kesh:0.13.0\|kesh-api\|kesh-api` | — | FILET |
| S15 image affichée par identifiant (**angle mort assumé**) | `running\|407fccb51e71\|kesh\|kesh-kesh-1` ⏎ `exited\|gcorbaz/kesh:0.13.0\|kesh-api\|kesh-api` | — | PASSE |
| S16 image voisine non Kesh | `running\|gcorbaz/kesh-autre:1\|autre\|y` ⏎ `exited\|gcorbaz/kesh:0.13.0\|kesh-api\|kesh-api` | — | PASSE |
| S17 démon injoignable | *(fichier vide)*, `FAUX_CODE=1` | — | ERREUR |
| **S18** scénario #619 réussi : service renommé, image officielle, **arrêté**, réglé | `exited\|gcorbaz/kesh:0.13.0\|kesh\|kesh-kesh-1` | `SAUVEGARDE_SERVICE=kesh` | PASSE |
| **S19** idem, avec l'orphelin `kesh-api` **arrêté** | `exited\|gcorbaz/kesh:0.13.0\|kesh\|kesh-kesh-1` ⏎ `exited\|gcorbaz/kesh:0.12.1\|kesh-api\|kesh-api` | `SAUVEGARDE_SERVICE=kesh` | PASSE |
| **S20** image Kesh d'un **autre** service, **à l'arrêt** | `exited\|gcorbaz/kesh:0.12.1\|ancien\|ancien` ⏎ `exited\|gcorbaz/kesh:0.13.0\|kesh-api\|kesh-api` | — | PASSE |
| **S21** état `dead`, image nue | `dead\|gcorbaz/kesh\|kesh-api\|kesh-api` | — | PASSE |
| **S22** scénario #619 seul, non réglé : renommé, image officielle, **en marche** | `running\|gcorbaz/kesh:latest\|kesh\|kesh-kesh-1` | — | FILET (refus dans la boucle, **avant** la sortie sur zéro) |
| **S23** image **nue** (sans étiquette) d'un autre service, en marche | `running\|gcorbaz/kesh\|autre\|z` ⏎ `exited\|gcorbaz/kesh:0.13.0\|kesh-api\|kesh-api` | — | FILET |

**Oracles de chaque cas** : code de sortie **non nul** ; motif de sa classe présent, motifs des autres
classes absents ; journal = **exactement une ligne**, qui commence par `ps --all` (forme de l'AC 5.2 ; ou
forme d'origine sous la seule mutation (O) — la forme exacte est tenue par G16, AC 7), **sans**
`INATTENDU` (anti-test-muet : un journal vide prouverait que le vrai `docker` a répondu — rouge) ;
`$W/garde/dossier/dump/.verrou` absent après le cas ; aucun `avant-restauration/` sous `$W/garde/dossier`.
S15 fixe l'angle mort au lieu de le taire : si un jour le filet lit les identifiants, ce cas change de
classe **délibérément**.

**Preuves de mutation** — chacune jouée en `RECETTE_SECTION=0`, sortie au Dev Agent Record, script
restauré **et** touché après chaque ; **la liste des cas rouges est exacte**, déroulée cas par cas à la
remédiation P3 sur les 23 cas (validation P2 R-M2 = F-M3 ; P3 F-4 pour les cas et mutations neufs) :

| Mutation de `kesh-restore.sh` | Cas rouges (exactement) | Verts |
|---|---|---|
| (O) garde d'origine (`git show fe162504:scripts/synology/kesh-restore.sh`, lignes 75-79) | S1, S2, S3, S4, S5, S7, S8, S11, S12, S13, S14, S22, S23 (13) | les 10 autres |
| (F) filet retiré (seul le service est gardé) | S3, S4, S14, S22, S23 (5) | les 18 autres |
| (F′) filet **sans** `case` d'état (tout `gcorbaz/kesh` d'un autre service refusé) | S19, S20 (2) | les 21 autres |
| (Z) sortie sur zéro conteneur retirée | S1, S5 (2) | les 21 autres |
| (I) contrôle « illisible » retiré | S11, S12 (2) | les 21 autres |
| (R) `SAUVEGARDE_SERVICE` ignoré (`kesh-api` en dur, comparaison **et** compteur) | S2, S4, S18 (3) | les 20 autres |
| (D) `dead` retiré de la liste fermée | S21 (1) | les 22 autres |
| (T) filet à étiquette obligatoire (`gcorbaz/kesh[:@]`) | S23 (1) | les 22 autres |

Déroulé de (O), qui sert de contrôle : la garde d'origine ignore le réglage et ne lit que l'étiquette
`kesh-api` ; elle passe sur S1, S2, S3, S5, S11, S12, S14, S22, S23 (motif PASSE au lieu de celui attendu)
et refuse S4, S7, S8, S13 avec son ancien libellé « un conteneur kesh-api est « … » », qui ne porte ni
`(service «` ni `image Kesh` ; S18-S21 passent, comme attendu. Sous (R), S19 reste vert : l'orphelin
`kesh-api` arrêté est compté (c'est S18, sans orphelin, qui attrape la mutation).

**Ce que la section ne prouve pas, écrit** : le comportement du **vrai** `docker ps --format` (séparateur,
`{{.Label …}}` sur un conteneur sans étiquette) — relevé au poste à la remédiation P1
(`running|407fccb51e71|mariadb|kesh-mariadb-dev`) et rejoué par la section 4 existante, qui fait tourner la
garde nouvelle contre de vrais conteneurs ; les sections 5 à 9 la font passer sur un `kesh-api` arrêté
(et, en prime, à côté du conteneur `kesh-recette-synology-db`, hors compose, étiquette vide réelle).
**Seul changement aux sections réelles** (validation P3, R-L6) : `recette.sh:153` exige en plus le motif
`est « running »` et `recette.sh:162` le motif `est « paused »` (comme `:159` exige déjà
`est « restarting »`), pour qu'un refus d'une autre cause ne les mette pas au vert.

**Prérequis de la recette réelle, écrit dans `docs/testing.md`** (pas de pré-vol, décision C-26-3-13) :
les sections 4 à 9 font tourner la garde nouvelle sur **tout l'hôte** ; elles refusent si l'hôte porte,
hors du projet de recette, un conteneur **actif** (`running`, `restarting`, `paused`…) du service
`kesh-api` — dépendance déjà vraie avec la garde d'origine — **ou**, dépendance **neuve** du filet, un
conteneur actif d'image `gcorbaz/kesh`, quel que soit son service (validation P3, R-L9). Le message de la
garde **nomme** le conteneur en cause : le rouge se diagnostique seul.

**Observation laissée à l'arbitrage** (validation P3, R-L9) : la section 0 n'a plus besoin de Docker ; un
test Rust qui lancerait `bash kesh-restore.sh` sous ce faux la ferait tourner dans le gate. Non requis par
les issues, **non prescrit ici** : signalé au Project Lead (la recette reste hors gate, comme depuis la
15-14b).

Autres contraintes : `docs/testing.md` § *Recette des scripts de sauvegarde Synology* (`:459`-) décrit la
section 0, le faux `docker`, `RECETTE_SECTION=0`, le prérequis ci-dessus et les limites ; la recette se
lance depuis le worktree, **hors gate**.

### AC 7 — G16 lit la garde nouvelle (#619, C-26-3-5)

`crates/kesh-api/tests/configuration_transmise.rs`, `synology_sauvegarde_la_base_par_le_dump` :

1. la regex de l'étape « refus Kesh actif » (`:3121`-`:3124`, qui exige aujourd'hui
   `--filter label=com.docker.compose.service=kesh-api`) suit la forme nouvelle : `docker ps --all`
   avec `{{.State}}`, `{{.Image}}` et l'étiquette de service, **indifférente au séparateur** (elle
   n'encode pas le `|` : c'est la clause 7.5 qui le tient, pour que la mutation (e) ne touche qu'elle —
   validation P2, F-M5) ; l'**ordre** des étapes reste contrôlé ;
2. clauses neuves : défaut de `SAUVEGARDE_SERVICE` = un service de `docker-compose.prod.yml`
   (`services_prod`) ; sortie sur zéro conteneur du service (`exit 1` sous la condition) ; filet par
   l'image `gcorbaz/kesh` présent ;
3. (g) : l'égalité des ensembles de réglages manuel/scripts inclut `SAUVEGARDE_SERVICE` ; le manuel le
   cite sous la forme `SAUVEGARDE_SERVICE (kesh-api` suivie d'une virgule ou d'une parenthèse ; la
   boucle « mêmes défauts dans les deux scripts » **n'inclut pas** ce réglage (le dump ne le lit pas) —
   l'écrire en commentaire ;
4. la doc-comment de G16 (`:2866`-`:2934`) et ses angles morts sont mis à jour (C-26-3-3, angle mort
   de l'image construite localement) ;
5. clause neuve sur la lecture : le `--format` porte le séparateur `|` et la boucle `IFS='|'` (AC 5.2) ;
   un séparateur blanc (`\t`, espace) rougit ;
6. **preuve de mutation**, chacune rejouée et consignée avec **l'ensemble exact** des clauses qui
   doivent rougir — « elle seule » est abandonné, deux mutations touchant légitimement deux clauses
   (validation P2, R-M3 = F-M5) :

   | Mutation | Clauses rouges attendues (et aucune autre) |
   |---|---|
   | (a) remettre la **seule ligne** `docker ps` d'origine (réglage `SAUVEGARDE_SERVICE` toujours déclaré) | étape « refus Kesh actif » (7.1) ; lecture à séparateur `|` (7.5) |
   | (b) retirer la sortie sur zéro conteneur | « zéro conteneur » (7.2) |
   | (c) retirer le filet par l'image | « filet » (7.2) |
   | (d) retirer `SAUVEGARDE_SERVICE` du manuel | binaire `configuration_transmise` : (g) égalité des ensembles ; (g) citation `SAUVEGARDE_SERVICE (kesh-api` (7.3) — et, dans le binaire `textes_coherents`, la positive de G20 sur `admin-manual.tex` (AC 9 ; validation P3, F-10) |
   | (e) remplacer `|` par une tabulation dans le `--format` et l'`IFS` | lecture à séparateur `|` (7.5) seule |

   G16 agrège ses erreurs (`echouer_si`, `configuration_transmise.rs:3466`) : copier **toutes** les
   lignes d'erreur de chaque run et constater qu'elles sont exactement celles du tableau. Restaurer et
   `touch` après chaque mutation (cargo garde sinon le binaire muté).
7. La clause existante « aucune commande `docker compose` » (`(?m)(?:^\s*|[;&|]\s*)docker compose\b`)
   doit rester verte : les libellés qui citent `docker compose up --no-start` sont des chaînes sur une
   seule ligne de code, jamais en tête de ligne ni après `;`, `&`, `|` hors guillemets — **attention**
   au `|` : le libellé de l'AC 5.4 ne doit contenir aucun `|` suivi de `docker compose` (validation P1,
   LOW-4).

8. **G14 lit `up`** (validation P3, F-9, C-26-3-19) : la commande neuve `docker compose up --no-start
   kesh-api` est la voie unique de recréation ; G14 (`configuration_transmise.rs:2689`, verbes
   `exec|stop|start|restart|logs|pull|rm`) l'ignore. Ajouter `up` aux verbes, et accepter les options
   longues (`--no-start`, `--force-recreate` : `-{1,2}[A-Za-z][A-Za-z-]*`) ; mettre à jour la doc-comment
   (« Formes non lues ») ; **inventaire avant la modification** :
   `grep -nE "docker compose.* up\b" docs/manual/fr/admin-manual.tex` — chaque `up` suivi d'un nom de
   service doit nommer un service du bon compose (prod dans les sections Synology, standard ailleurs) ;
   un `up -d` sans service reste non lu (pas de nom après les options). Relevé à la remédiation P3 :
   **6** lignes nomment un service (`kesh-api` : `:1289`, `:1296`, `:1348`, `:1927`, `:2565`, `:2572`),
   toutes hors sections Synology, `kesh-api` existant dans `docker-compose.yml` — G14 reste verte. Mutation : écrire
   `docker compose up --no-start kesh` dans la section Synology → G14 rouge, sur cette ligne.

G14 (`les_services_cites_existent`) reste verte : les commandes `docker compose stop|start kesh-api`
et `up --no-start kesh-api` du manuel sont écrites avec le service livré.

### AC 8 — Le manuel d'administration dit le compose adapté (#619, C-26-3-4, C-26-3-7)

Dans `docs/manual/fr/admin-manual.tex` :

1. `:1671` — les réglages : ajouter `\texttt{SAUVEGARDE\_SERVICE} (\keshcommand{kesh-api})`, avec ce
   qu'il désigne (le service de Kesh dans votre compose, lu par `kesh-restore.sh` seul) ; dire de
   `SAUVEGARDE_RESEAU` qu'il désigne un réseau Docker **existant**, non `internal`, depuis lequel
   l'**adresse de la base est routable** — sur Synology, la base est le paquet MariaDB de DSM, **hors
   Docker**, jointe par son adresse IP LAN (`host` du `[client]`) ; ce qui fait échouer `frontend` est son
   inexistence, non la position de la base ; le plus simple : **le réseau de `kesh-api`**, qui l'atteint
   déjà (validation P3, F-11).
2. Un paragraphe **« Compose adapté »** (reverse proxy, service renommé) : réseau et service suivent le
   compose réellement déployé ; commande qui affiche les réseaux du conteneur, écrite pour **le nom de
   votre conteneur** (`kesh-api` est le `container_name` du compose livré ; un compose adapté peut en
   porter un autre — `docker ps --format '{{.Names}}'` le liste ; validation P1, R-11) :
   `docker inspect -f '{{range $n, $_ := .NetworkSettings.Networks}}{{$n}} {{end}}' <conteneur>`, à
   vérifier à l'exécution et à recopier dans un `lstlisting` (accolades : `\{`, ou listing) ; rappel qu'un réseau **déclaré dans le compose** porte le préfixe du projet
   (`<projet>_kesh-internal` pour la recette Traefik du manuel, `\S\ref{sec:traefik}`) ; exemple d'un
   reverse proxy sur le réseau `proxy`, pour **la tâche planifiée** et **le rechargement** :
   ```sh
   SAUVEGARDE_RESEAU=proxy bash /volume1/docker/kesh/kesh-dump.sh
   SAUVEGARDE_RESEAU=proxy SAUVEGARDE_SERVICE=kesh SAUVEGARDE_BASE=kesh \
       bash /volume1/docker/kesh/kesh-restore.sh <dossier dump restauré>
   ```
   (forme exacte libre, mais G16 (f) exige toujours la commande nue `bash /volume1/docker/kesh/kesh-dump.sh`
   et `SAUVEGARDE_BASE=… bash /volume1/docker/kesh/kesh-restore.sh <dossier dump restauré>` : les garder).
3. `:1744` — étape 1 du déroulé : le script refuse si un conteneur du service `SAUVEGARDE_SERVICE` ou
   de l'image Kesh est actif, et **refuse aussi s'il ne trouve aucun conteneur de ce service** ; plus
   de « quel que soit le nom du projet » seul (vrai, mais il laissait croire au service quelconque).
4. *Recovery depuis Snapshot*, étapes d'arrêt et de redémarrage (`:1723`-`:1738`) : « le service de
   Kesh (`kesh-api` dans le compose livré) » ; la commande reste écrite avec `kesh-api` (G14, G16 (f)).
   **Et** (validation P1, MEDIUM-4, C-26-3-11) : si `docker compose ps -a` ne montre plus le conteneur
   (après un `docker compose down`, ou sur un NAS neuf — cas de la *Stratégie 3-2-1*, `:1788`), le
   recréer sans le démarrer, `docker compose up --no-start kesh-api` — même commande que le message
   du script ; elle lit le compose entier : `.env` doit être en place. `:1788` (« arrêt de
   `kesh-api`, `kesh-restore.sh`, redémarrage ») renvoie à cette étape (R-12).
5. Listing de suppression d'une base illisible (`:1767`) : **inchangé** (`--network frontend`, que la
   recette réécrit, `recette.sh:280`) ; la prose qui le précède dit de remplacer `frontend` par la
   valeur de `SAUVEGARDE_RESEAU`.
6. `:572` (§ Synology, réseau `frontend`) : « C'est aussi par lui que les conteneurs de sauvegarde
   atteignent la base » est réécrit : les conteneurs de sauvegarde s'y **rattachent** pour joindre
   l'adresse de la base (MariaDB de DSM, hors Docker) ; renvoi au réglage si `kesh-api` vit sur un autre
   réseau (F-11).
7. `:1669` (C-26-3-7) : la phrase sur `main` dit que l'URL désigne l'étiquette de la version
   installée, à garder égale à l'image du compose. Le `v0.13.0` des listings **ne change pas** ici
   (release v0.13.1, G19).
8. Sections en `\sloppy` si un débordement apparaît (C-15-14-75) ; `Overfull \hbox` relevés au journal
   de compilation et consignés.

### AC 9 — Une garde de texte interdit le retour des deux phrases (C-26-3-5)

Dans `crates/kesh-api/tests/textes_coherents.rs`, une garde **G20** (numéro à vérifier au merge), au
tableau d'en-tête :

**Ciblée sur la forme du défaut, sensible à la casse** (validation P1, R-1 = HIGH-2, C-26-3-9) :
l'expression « son propre compte de banque » est **juste** et figure à six endroits des manuels
(`user-manual.tex:245`, `:1166`, `:1492`, `:1580`, `:1755`, `admin-manual.tex:2472`) et dans six
messages du catalogue `fr-CH` (`messages.ftl:30`, `:34`, `:35`, `:791`, `:1178`, `:2035` — recompté en P4 ciblée, L4) ; une garde
insensible à la casse rougirait sur du code correct, et serait « réparée » en devenant muette.

- **négatives**, sur la source `.tex` **brute** (pas de `normaliser`, qui ne garantit pas les
  guillemets) des trois manuels français, recherche de sous-chaîne **sensible à la casse** :
  expressions régulières, `\s+` entre les mots pour qu'un **retour à la ligne de source** ne les
  contourne pas (validation P2, R-L5 = F-L3), **`[\s~]+`** pour que l'espace insécable LaTeX `~` ne les
  contourne pas non plus (validation P3, F-8) : `\\(?:textbf|emph)\{Compte[\s~]+de[\s~]+banque\}`,
  `«[\s~]*Compte[\s~]+de[\s~]+banque[\s~]*»` (`\s` Unicode couvre U+00A0 et U+202F ; le manuel
  n'emploie ni `\og` ni `\enquote`, relevé P2), `[Tt]outes[\s~]+les[\s~]+banques[\s~]+suisses` — qui
  interdit aussi « toutes les banques suisses ne le proposent pas » : la phrase de l'AC 3.1 emploie
  « la plupart … pas toutes », à écrire telle (R-L8 iv). Faux positifs revérifiés en P3 : aucun sur les six
  « propre compte de banque » (ni `\textbf`, ni `\emph`, ni guillemets). Hors couverture, assumé :
  `\textit{…}`, `\keshpath{…}`, `\enquote{…}` (aucun employé) ;
- **négatives sur les catalogues** : dans **chacun des quatre** (`fr-CH`, `de-CH`, `en-CH`, `it-CH`),
  la valeur de `bank-import-labels-drop-zone` contient `CSV`, et celle de
  `bank-import-labels-page-title` ne contient pas `CAMT.053` (validation P1, R-10). `catalogue_fr()`
  (`textes_coherents.rs:98`) ne lit que `fr-CH` (LOW-2) : la garde le généralise en
  `catalogue(locale)` (même lecteur, continuation de lignes comprise) ou en écrit un second, et
  **asserte que chaque catalogue lu porte plus de 1000 clés** (anti-test-muet) ;
- **positives adossées, même fichier** : `user-manual.tex` porte `propre compte de banque` (la forme
  juste existe : la négative n'est pas muette par lecture vide) et `Compte comptable lié` — **c'est
  elle qui tient la phrase de l'AC 1.2** (absente aujourd'hui) ; `Comptes bancaires`, déjà présente
  (`:208`, `:229`, `:2577`), ne prouve que la lecture (R-L5 iii) ; les positives aussi en `[\s~]+` entre
  les mots (R-L8 i) ; `admin-manual.tex` porte `SAUVEGARDE_RESEAU=proxy` (listing) et
  `SAUVEGARDE_SERVICE=` (listing, source brute) — la **prose** `\texttt{SAUVEGARDE\_SERVICE}` est tenue
  par G16 (g), non par G20 (R-L8 ii) ; chaque catalogue porte les deux clés ;
- preuves de mutation, une par négative : réintroduire `\textbf{Compte de banque}`, puis la même
  coupée par un retour à la ligne (`\textbf{Compte de⏎banque}`), puis `marqué « Compte de banque »`,
  puis `Toutes les banques suisses`, puis la valeur `de-CH` d'origine de la zone de dépôt, puis la valeur
  `en-CH` d'origine du titre (`CAMT.053 bank import`) → rouge chaque fois ; et vérifier qu'écrire « son propre compte de banque » une fois
  de plus **ne** rougit **pas**. Restaurer, `touch`.

### AC 10 — PDF régénérés et contrôlés (C-26-3-6)

1. `scripts/mem-guard.sh make -B -C docs/manual admin user` ; PDF commités avec leurs sources ; la
   brochure n'est pas régénérée (non touchée).
2. Par `occ` (en tête des AC ; `occ` est sensible à la casse), PDF d'avant ≥ 1 — valeurs relevées
   en remédiation P1 sur `fe162504` entre parenthèses — puis PDF neuf = **0** :
   user : `Compte de banque` (2), `marqué « Compte de banque »` (1),
   `fourni par toutes les banques suisses` (1), `Nouvel import CSV` (1),
   `Importer → Nouvel import` (2), `Cliquer sur un compte ouvre son détail` (1), `Bilan / Résultat`
   (1), `Devise : CHF par défaut` (1), `une fois des écritures saisies` (1),
   `Import bancaire, puis l'import` (2), `Multibanking ou Klippa` (1) ;
   admin : `quel que soit le nom du projet` (1, si la phrase est réécrite).
   **Pas** de contrôle d'absence sur `SAUVEGARDE_RESEAU (frontend` : G16 (g) **exige** cette forme au
   manuel (`configuration_transmise.rs`, `{var} \({défaut}[,)]` ; validation P1, LOW-1) — elle reste.
   PDF neuf ≥ 1 : `Compte comptable lié`, `Confirmer l'import`, `Importer des relevés`,
   `SAUVEGARDE_SERVICE`, `SAUVEGARDE_RESEAU=proxy`, `up --no-start`, et la phrase neuve sur le
   CAMT.053 (motif choisi au dev). Toutes les valeurs au Dev Agent Record, commande comprise.
3. Aucun `??` de référence non résolue (`\ref{sec:import-csv}`) : `pdftotext … | grep -c '??'` sur les
   deux PDF, avant/après.

### AC 11 — CHANGELOG et propagation (C-26-3-8)

1. `CHANGELOG.md` : section `## [0.13.1] — Non publié` (créée si absente, au-dessus de `[0.13.0]`,
   forme exacte que lit `scripts/prepare-release.sh:237`), entrées sous `### Corrigé` pour #616, #617,
   #619, et une ligne **« Action requise »** pour l'exploitant Synology : re-télécharger
   `kesh-restore.sh` au tag de la version, poser `SAUVEGARDE_SERVICE` si son service ne s'appelle pas
   `kesh-api`, `SAUVEGARDE_RESEAU` s'il n'est pas sur `frontend`, recréer le conteneur par
   `docker compose up --no-start` s'il a été supprimé. Formulation : « refuse de commencer un
   rechargement tant que… », jamais « ne recharge jamais sous un Kesh en marche » (LOW-5).
   **Une seule section `[0.13.1]`** (validation P1, R-9, C-26-3-8 amendé) : les 26-1, 26-2 et 26-3
   l'écrivent chacune ; au rebase, les entrées se **fusionnent par union sous UNE section** (une seule
   en-tête `## [0.13.1] — Non publié`, un seul `### Corrigé`, etc.) — `prepare-release.sh` refuse
   la release dès qu'il en compte plus d'une (`NB_SECTIONS -gt 1`, `:237`-`:246`). Contrôle après
   chaque rebase et au dernier commit : `grep -c '^## \[0\.13\.1\] — ' CHANGELOG.md` → **1**, et
   `grep -c '^### ' ` dans cette section sans titre en double.
2. Grep du symptôme **par valeur** après chaque patch, **chaque commande bornée** et avec son résultat
   **attendu**, sites restants nommés (validation P2, R-M4 = F-M7, R-L8) — `B` désigne le périmètre
   **`-- . ':(exclude)_bmad-output'`** (artefacts BMAD et rétrospectives : ne se corrigent pas). ⚠️ La forme
   courte `':!_bmad-output'` est **refusée** par git 2.53 (« Spécificateur magique '_' non implémenté »,
   code 128) : la commande ne rend **rien**, ce qui se lit comme « 0 » — un vert muet (validation P3,
   R-M1 = F-1). **Chaque commande se rejoue avec son code de sortie** (`; echo "code $?"` : 0 ou 1, jamais
   128), et ne se passe à `wc -l` qu'après ce contrôle :
   | Commande | Attendu après la story |
   |---|---|
   | `git grep -nIF "Compte de banque" -- docs` | 0 |
   | `git grep -nIi "toutes les banques" B` | 0 |
   | `git grep -nI "Import bancaire CAMT.053" -- crates/kesh-i18n/locales frontend/src docs/i18n-glossaire.md docs/manual/fr/user-manual.tex docs/manual/fr/admin-manual.tex` | seulement `messages.ftl:1043` (commentaire de story) |
   | `git grep -nI "Import bancaire CAMT.053" B` | les sites ci-dessus **plus**, assumés (AC 2.4) : `README.md:41`, `marketing-brochure.tex:388` (noms de fonctionnalité), `errors.rs:644`, `:2032`, `routes/bank_imports.rs:1` (commentaires de story), `docs/kesh-specifications.txt:1534`, `:2458` (historique) |
   | commande de l'AC 2.4 (`emph{Importer}`, `emph{Import bancaire}`, `Quotidien → Importer**`, `Nouvel import`, sur `docs`) | 0 |
   | `git grep -nI "compose create" B` | 0 (une seule commande, `up --no-start`) |
   | `git grep -nI "service=kesh-api" B` | **1** : `scripts/synology/recette.sh:67` (`etat_kesh_api`, section 4 réelle, filtrée sur le projet de recette — reste) ; `kesh-restore.sh:76` et `configuration_transmise.rs:3123` disparaissent (AC 5.2, AC 7.1). **1 exactement** : le faux `docker` de la section 0 compare au service `<S>` passé en argument, jamais au littéral `service=kesh-api`, et `docs/testing.md` ne l'écrit pas non plus (validation P3, R-M1 = F-2 ; attendu rendu inconditionnel en P4 ciblée, L2) |
   | `git grep -nI "SAUVEGARDE_SERVICE" B` | script, manuel d'administration (+ PDF), G16, G20, recette, `docs/testing.md`, CHANGELOG — liste au Dev Agent Record |
   La colonne « attendu » se **recompte** au dernier commit ; un écart s'explique ou se corrige.
3. `README.md:50` (puce *Exploitation sur NAS Synology*) : relire ; ne modifier que si une phrase y
   devient fausse. `website/` : rien à modifier (AC 3, grep relevé) — à rejouer.

### AC 12 — Gates

- **Backend** : `scripts/test-fast.sh` (fmt + clippy + nextest), base de gate remise à zéro **avant**
  (règle inconditionnelle) — sur la base **propre au worktree** (`kesh_26_3`), jamais par
  `docker restart` de `kesh-mariadb-dev` (consignes, point 2). Le Rust touché est dans
  `crates/kesh-api/tests/` (G16, G20) : pas de `kesh-db`.
- **Frontend** : `npm run check`, `npm run lint-i18n-ownership`, `npm run test:unit`, `npm run build`.
- **E2E complet** au **dernier commit de code** (D7) — l'écran d'import est touché ; échecs jugés
  fichier par fichier contre `docs/testing.md` § *Les échecs attendus*.
- **Recette Synology** (`bash scripts/synology/recette.sh`) verte au dernier commit qui touche
  `scripts/synology/`, sortie complète au Dev Agent Record.
- Ne déclarer que ce qui a tourné, avec son périmètre ; recompter tout décompte depuis la source.

## Tasks / Subtasks

- [x] **T0 — Relevé** (AC tous)
  - [x] Relancer chaque commande d'inventaire des AC sur l'état courant (`git log --oneline -1`), noter
        les écarts avec `fe162504` (les 26-1/26-2 mergées entre-temps décalent les lignes).
  - [x] `occ` sur les PDF d'avant, valeurs ≥ 1 consignées (AC 10).
  - [x] Créer la base du worktree et le port attribué par l'orchestrateur (consignes, points 6 et 8).
- [x] **T1 — Manuel utilisateur** (AC 1, 2, 3)
  - [x] § Ajout / Modification d'un compte sur l'inventaire des dialogues ; phrase « compte bancaire ».
  - [x] § Import dans Kesh, § Import avec un profil existant ; `\label{sec:import-csv}`.
  - [x] `:1638` reformulé (plupart, CSV, contrôle de solde à la main, MT940 non lu).
- [x] **T2 — Écran d'import** (AC 4) : 4 catalogues, `aria-label`, titre et `<title>`, replis.
- [x] **T3 — `kesh-restore.sh`** (AC 5) : réglage, lecture unique, filet image, sortie fermée, en-tête.
- [x] **T4 — Recette** (AC 6) : section 0 contre le faux `docker` (fonction définie et exportée dans
      le sous-shell de chaque cas), `RECETTE_SECTION=0`, cas S1-S23, huit mutations (O, F, F′, Z, I, R, D,
      T) aux cas rouges exacts, assertion `type -t docker` = `file`, motifs `est « running »` /
      `est « paused »` aux lignes 153/162, `docs/testing.md` (section 0, prérequis de la recette réelle).
- [x] **T5 — G16** (AC 7), **G14 lit `up`** (AC 7.8) et **G20** (AC 9), avec leurs mutations.
- [x] **T6 — Manuel d'administration** (AC 8).
- [x] **T7 — PDF** (AC 10), **CHANGELOG** et propagation (AC 11).
- [x] **T8 — Gates** (AC 12), commit par étape BMAD, messages `refs #616 #617 #619`.

## Dev Notes

### Ce que la story change, ce qu'elle doit préserver

- **`kesh-restore.sh`** (127 lignes) — aujourd'hui : refus si un conteneur étiqueté
  `com.docker.compose.service=kesh-api` est dans un état hors `exited|created|dead`, sur tout l'hôte ;
  puis verrou, empreinte, nom de la base, sonde à trois états, dump de sécurité, rechargement, reprise.
  **Ne change que l'étape 1** (et les textes qui la décrivent). À préserver : l'ordre des étapes, le
  verrou pris **après** la garde (la garde ne doit rien écrire), `set -euo pipefail` et `umask 077`
  avant le premier `docker run`, l'absence de toute commande `docker compose` dans le script (G16 (c) :
  `(?m)(?:^\s*|[;&|]\s*)docker compose\b` — les **messages** peuvent citer `docker compose up
  --no-start`, sur une seule ligne de code, jamais une ligne de commande), `$1` employé une seule fois
  (`SOURCE=$(cd "$1" && pwd -P)`).
- **Piège `set -e` et `while read`** : une boucle alimentée par `<<< "$LISTE"` tourne dans le shell
  courant (un `exit 1` y sort bien du script) ; un `docker ps … | while read` tournerait dans un
  sous-shell, où `exit 1` ne sortirait que du sous-shell — **garde ouverte**, exactement le défaut à
  corriger. Le compteur de conteneurs du service doit lui aussi survivre à la boucle.
- **`{{.Image}}`** affiche l'image **telle que référencée à la création** (`gcorbaz/kesh:latest`,
  `gcorbaz/kesh:0.13.0`, éventuellement `docker.io/gcorbaz/kesh:…`) — ou un identifiant si l'étiquette
  a été retirée : d'où l'angle mort écrit (C-26-3-3). **Constaté** à la remédiation P1 sur le poste
  de dev : `docker ps --format '{{.State}}|{{.Image}}|{{.Label "com.docker.compose.service"}}|{{.Names}}'`
  rend `running|407fccb51e71|mariadb|kesh-mariadb-dev` — image affichée par son identifiant. La forme
  du `--format` (avec `{{.Label "…"}}`) est donc valide sur le poste (Docker Compose 2.40.3).
- **G16 est un couplage écrit** : sa doc-comment le dit (« une écriture équivalente rougirait »). La
  modifier fait partie de la story, pas d'un écart.
- **La recette** : la section 0 (AC 6) n'appelle jamais le vrai `docker` ; ses cas ne dépendent ni de
  l'hôte ni des sections réelles. Piège à ne pas retomber dedans : un faux exécutable en tête de `PATH`
  est **masqué** par `kesh-restore.sh:54` (`export PATH="/usr/local/bin:/usr/bin:/bin:…"`) — d'où la
  fonction exportée (C-26-3-14). Les sections 1 à 9, réelles, ne changent pas.

### Conflits attendus avec les Stories 26-1 et 26-2 (développées en parallèle)

| Fichier | 26-1 | 26-2 | 26-3 |
|---|---|---|---|
| `docs/manual/fr/user-manual.tex` | § Profils banque, import CSV | `\section{Comptes bancaires}\label{sec:comptes-bancaires}` (renvoi AC 2.6) | § Ajout d'un compte, § Import dans Kesh, `:1638`, § Import avec un profil existant |
| `BankImportUpload.svelte` | profil CSV ? | libellé des options du sélecteur « Compte bancaire cible » (`{account.bankName} ({account.iban})`, `:212`) | `aria-label` `:232` |
| catalogues `messages.ftl` | clés du profil | clés de la description | `bank-import-labels-drop-zone`, `-page-title` |
| `CHANGELOG.md` | `[0.13.1]` | `[0.13.1]` | `[0.13.1]` — **une seule section** après fusion (AC 11.1) |
| `textes_coherents.rs` | garde ? | garde ? | G20 |
| PDF des manuels | régénéré | régénéré | régénéré |

Les PDF ne se fusionnent pas : **le dernier rebasé les régénère** depuis les sources fusionnées. Le
`\label{sec:import-csv}` : un seul, s'il est déjà posé par la 26-1, le réutiliser.

### Previous story intelligence (15-14b, scripts Synology)

- La garde a été **simplifiée à la racine** en revue P4 (C-15-14-77 : le script ne touche plus à
  Kesh) après un recyclage de défauts ; puis élargie aux états `restarting`/`paused` (C-15-14-81). Ne
  pas réintroduire de `stop`/`up` dans le script.
- L'oracle de la recette est l'**empreinte de contenu** (C-15-14-79), pas `CHECKSUM TABLE`.
- La recette extrait les listings **du manuel** (`listing()`, `recette.sh:76`) : un listing dupliqué
  (deux `lstlisting` contenant le même motif) fait échouer son `assert len(b) == 1`. Le paragraphe
  « Compose adapté » de l'AC 8 ne doit pas reproduire `[client]`, `CREATE USER 'kesh'@'%'`,
  `TO 'kesh_backup'` ni `DROP DATABASE <base>` dans un listing.
- La 15-14b a convergé en sept passes de revue, dont plusieurs nées d'une remédiation : la passe
  ciblée sur la dernière remédiation est attendue en fin de boucle.

### Git intelligence (5 derniers commits)

`fe162504` kickoff Epic 26 (planning) ; `46ae1804` release v0.13.0 (macros des manuels, G19 sur
`v0.13.0`) ; `c8016a7b` rétrospective Epic 15 ; `d2460a2d` 15-1c-ii (lettrage, manuel utilisateur
réécrit en partie — numéros de ligne du manuel décalés depuis) ; `8a2c827e` `prepare-release.sh`.
Aucun ne touche `scripts/synology/` depuis la 15-14b.

### Technique

Pas de bibliothèque neuve. `docker ps --format` avec `{{.Label "…"}}` vérifié sur le poste (Docker
Compose 2.40.3, sortie relevée plus haut). **Une seule commande pour recréer un conteneur sans le
démarrer : `docker compose up --no-start <service>`** (C-26-3-11), dans le message du script et au
manuel — `docker compose create` n'est employé nulle part hors `_bmad-output/` (grep borné de
l'AC 11.2 : 0). `export -f` est propre à bash : les deux scripts et la recette sont lancés par `bash`.

### Project Structure Notes

- Scripts d'exploitation : `scripts/synology/` (téléchargés par l'exploitant au tag de version).
- Gardes de texte : `crates/kesh-api/tests/textes_coherents.rs` (G2…G19) ; gardes de configuration :
  `configuration_transmise.rs` (G14…G17). Utilitaires partagés : `crates/kesh-api/tests/common/manuel.rs`
  (`normaliser`, `sections_synology`, `listings`).
- Manuels : seules les sources FR existent (`docs/manual/{de,en,it}/` = `README.md`).

### References

- [Source: _bmad-output/planning-artifacts/epic-26-recette-v0-13.md § 26-3]
- [Source: issues #616, #617, #619 et leur commentaire de décision du 2026-10-10]
- [Source: scripts/synology/kesh-restore.sh:52-79 ; recette.sh:31-90, 152-178, 280]
- [Source: crates/kesh-api/tests/configuration_transmise.rs:2866-3420 (G16), :2650-2720 (G14)]
- [Source: crates/kesh-api/tests/textes_coherents.rs:1-24, :1099 (G19)]
- [Source: docs/manual/fr/admin-manual.tex:566-572, 1657-1775 ; user-manual.tex:333-350, 1628-1690]
- [Source: frontend/src/routes/(app)/accounts/+page.svelte:527-660 ; bank-accounts/+page.svelte:95-115, 337-372 ; bank-import/+page.svelte ; lib/features/bank-import/BankImportUpload.svelte:200-265]
- [Source: _bmad-output/implementation-artifacts/epic-15-choix-autonomes.md C-15-14-75, 77, 79, 81]
- [Source: _bmad-output/implementation-artifacts/15-14a-manuels-et-libelles.md:52-76 (`occ`)]
- [Source: CLAUDE.md § Test Locally First, § Review Iteration Rule (le prompt nomme le manuel ; PDF aplati), § Synchroniser TOUTES les docs]

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5 (développement, en autonomie).

### Debug Log References

- Gate backend : `kesh-gate-logs/26-3-gate-dev.log` ; E2E : `kesh-gate-logs/26-3-e2e-dev.log` ;
  recette Synology complète : `kesh-gate-logs/26-3-recette-dev.log`.
- Premier gate backend rouge (27 échecs `Unknown database '_sqlx_test_…'`) : le gate de la 26-2 tournait
  au même moment sur le même serveur MariaDB, et le ménage des bases `_sqlx_test_*` de `sqlx::test`
  efface celles de l'autre run. Relancé seul, après la fin du sien : vert. **À signaler** : deux gates
  complets simultanés sur `kesh-mariadb-dev` se détruisent mutuellement.

### Completion Notes List

**Relevé T0** (sur `f00aa6c0`, `main` = `9b724027`) : les numéros de ligne des AC sont ceux de
`fe162504` (aucune des 26-1, 26-2 n'est mergée). Commande de l'AC 2.4 : 7 lignes, comme prévu. **26-1
non mergée** : § *Profils banque* non touchée, non vérifiée (AC 2.7, à signaler). **26-2 non mergée** :
renvois au texte, pas à `sec:comptes-bancaires` (AC 2.6, C-26-3-23).

**Écart à la fiche, consigné** : C-26-3-22 (la boucle lit la ligne entière puis la découpe, trois `|`
exactement — la forme littérale de l'AC 5.2 laissait passer `|||` et `a|b|c|d|`).

**Recette, section 0** (`RECETTE_SECTION=0 bash scripts/synology/recette.sh`) : 23 cas S1-S23 verts,
« RECETTE VERTE ». **Mutations de `kesh-restore.sh`**, chacune jouée en section 0, script restauré et
touché après chaque, cas rouges relevés par `grep -oE 'ÉCHEC S[0-9]+'` :

| Mutation | Rouges observés | Attendu (AC 6) |
|---|---|---|
| (O) garde d'origine `fe162504:75-79` | S1 S2 S3 S4 S5 S7 S8 S11 S12 S13 S14 S22 S23 (13) | idem |
| (F) filet retiré | S3 S4 S14 S22 S23 (5) | idem |
| (F′) filet sans `case` d'état | S19 S20 (2) | idem |
| (Z) sortie sur zéro retirée | S1 S5 (2) | idem |
| (I) contrôle « illisible » retiré | S11 S12 (2) | idem |
| (R) `kesh-api` en dur | S2 S4 S18 (3) | idem |
| (D) `dead` retiré (deux `case`) | S21 (1) | idem |
| (T) filet à étiquette obligatoire | S23 (1) | idem |

**Recette complète** (vrai Docker, poste de dev, au commit `94f7e854`) : « RECETTE VERTE », 111 lignes
`OK`, 0 `ÉCHEC` ; sections 4 : refus `est « running »`, `est « restarting »`, `est « paused »` nommant
`kesh-recette-synology-kesh-api-1` ; l'hôte portait 21 conteneurs dont plusieurs hors compose à étiquette
vide (`created|debian:bookworm-slim||kesh1513b-t0-probe`) : lus sans refus. Assertion `type -t docker` =
`file` verte après la section 0 et en tête de la section 1.

**Mutations des gardes Rust** (binaires `configuration_transmise` et `textes_coherents`, restauration et
`touch` après chaque ; lignes d'erreur copiées) :

- (a) ligne `docker ps` d'origine → G16 : « sans refus « Kesh actif » » + « lecture à séparateur `|` » (2) ;
- (b) sortie sur zéro retirée → G16 : « zéro conteneur … ne fait pas sortir » (1) ;
- (c) filet retiré → G16 : « le filet par l'image gcorbaz/kesh manque » (1) ;
- (d) `SAUVEGARDE_SERVICE` retiré du manuel → G16 : (g) égalité des ensembles + (g) citation ; G20 :
  positive `SAUVEGARDE_SERVICE=` (3, deux binaires) ;
- (e) `|` → tabulation dans le `--format` et l'`IFS` → G16 : « lecture à séparateur `|` » seule ;
- G14 : `docker compose up --no-start kesh` en section Synology → « :1746 (section Synology) : `up kesh`,
  service absent de docker-compose.prod.yml » ;
- G20 : `\textbf{Compte de banque}` (:338), la même coupée par un retour à la ligne (:338), `marqué
  « Compte de banque »` (:346), `Toutes les banques suisses` (:1649), valeur `de-CH` d'origine de la zone
  de dépôt, valeur `en-CH` d'origine du titre → rouge chaque fois, sur le seul G20 ; « son propre compte
  de banque » ajouté une fois de plus → **vert** (aucun faux positif).

**G14** — inventaire `grep -nE "docker compose.* up\b"` avant modification : 7 lignes nomment un service
(`kesh-api` : 1289, 1296, 1348, 1746 — neuve, Synology —, 1947, 2585, 2592) ; G14 verte.

**PDF** (`scripts/mem-guard.sh make -B -C docs/manual admin user`, puis `occ` sous `LC_ALL=C.UTF-8`) :
avant (`fe162504`) → après : `Compte de banque` 2 → 0 ; `marqué « Compte de banque »` 1 → 0 ;
`fourni par toutes les banques suisses` 1 → 0 ; `Nouvel import CSV` 1 → 0 ; `Importer → Nouvel import`
2 → 0 ; `Cliquer sur un compte ouvre son détail` 1 → 0 ; `Bilan / Résultat` 1 → 0 ; `Devise : CHF par
défaut` 1 → 0 ; `une fois des écritures saisies` 1 → 0 ; `Import bancaire, puis l'import` 2 → 0 ;
`Multibanking ou Klippa` 1 → 0 ; admin `quel que soit le nom du projet` 1 → 0 ; `SAUVEGARDE_SERVICE`
0 → 6. Présents au neuf : `Compte comptable lié` 2, `Confirmer l'import` 4, `Importer des relevés` 4,
`La plupart des banques suisses le proposent, pas toutes` 1, `MT940 n'est pas lu` 1, `contrôle du solde
se fait alors à la main` 1, `SAUVEGARDE_RESEAU=proxy` 2, `up --no-start` 1, `SAUVEGARDE_RESEAU
(frontend` 1 (exigé par G16 (g)). `??` : 0 et 0 sur les deux PDF, avant et après. `Overfull \hbox` dans
les lignes touchées : un (user, `Récupération du fichier CAMT.053`, 5,3 pt) corrigé par `sloppypar` ;
zéro ensuite (deux `Underfull` admin, sans effet). Contrôle visuel aplati : listings `docker inspect` et
compose adapté rendus avec leurs accolades.

**AC 11.2, rejoué au commit `94f7e854` avec code de sortie** : `Compte de banque` (docs) 0 ligne, code 1 ;
`toutes les banques` (B) 0, code 1 (après reformulation du CHANGELOG et de G20, qui la citaient) ;
`Import bancaire CAMT.053` (périmètre restreint) : `messages.ftl:1043` seul, code 0 ; (B) : README:41,
`errors.rs:644`, `:2032`, `bank_imports.rs:1`, `messages.ftl:1043`, `kesh-specifications.txt:1534`,
`:2458`, brochure:388 — les sites assumés, code 0 ; AC 2.4 : 0, code 1 ; `compose create` : 0, code 1 ;
`service=kesh-api` : **1** (`recette.sh:186`, `etat_kesh_api` — décalé de :67 par la section 0), code 0 ;
`SAUVEGARDE_SERVICE` : CHANGELOG, `configuration_transmise.rs`, `textes_coherents.rs`, `admin-manual.tex`,
`kesh-restore.sh`, `recette.sh` (et `docs/testing.md` ne l'écrit pas), code 0. AC 3.2 : 4 lignes,
`user-manual.tex:1650` (la phrase neuve, « la plupart ») remplaçant `:1638`. README:50 relu : juste.
`website/` : rien d'universel (relu). `CHANGELOG.md` : `grep -c '^## \[0\.13\.1\] — '` → 1, un seul
`### Corrigé`.

**Vitest** : `i18n-keys.test.ts` rougissait (`sitesTotal` 2020 → 2022 : l'`aria-label` et le `<title>` de
l'écran d'import lus par clé) — compteur mis à jour, recompté par `grep -o "i18nMsg("` aux deux bornes
(`BankImportUpload.svelte` 21 → 22, `bank-import/+page.svelte` 2 → 3).

**Gates au commit de code `94f7e854` + compteur Vitest** (aucun code modifié depuis, hors fiche) :
`cargo fmt --check` vert ; `clippy -D warnings` vert ; `scripts/test-fast.sh --ci` sur `kesh_263` remise à
zéro : **3341 passés, 0 échec, 5 ignorés** (210 s) ; `npm run check` 0 erreur (27 avertissements
préexistants) ; `lint-i18n-ownership` PASS ; `npm run test:unit` 1287 tests, **1 échec** au premier run
(compteur ci-dessus), `i18n-keys.test.ts` 11/11 après correction — *la suite Vitest complète est à
rejouer au dernier commit* ; `npm run build` vert ; **E2E complet** (port 3033, `kesh_e2e_263`
reconstruite, montage complet de `docs/testing.md`) : **256 passés, 7 échoués, 19 ignorés** — les 7 sont
exactement les KF-029 (#97) de la liste attendue (`mode-expert:26`, `:41`, `onboarding-path-b:65`, `:92`,
`onboarding:57`, `:77`, `:150`).

**Signalé, non prescrit** : la section 0 n'a plus besoin de Docker ; un test Rust qui la lancerait la
mettrait dans le gate (observation P3 R-L9).

- Spécification : analyse de contexte complète (issues, epic, scripts, gardes G14/G16/G19, écrans,
  catalogues, manuels) — fiche prête pour la validation.

- Spécification : analyse de contexte complète (issues, epic, scripts, gardes G14/G16/G19, écrans,
  catalogues, manuels) — fiche prête pour la validation.

### File List

- `scripts/synology/kesh-restore.sh`, `scripts/synology/recette.sh`
- `crates/kesh-api/tests/configuration_transmise.rs` (G14, G16), `crates/kesh-api/tests/textes_coherents.rs` (G20)
- `crates/kesh-i18n/locales/{fr-CH,de-CH,en-CH,it-CH}/messages.ftl`
- `frontend/src/lib/features/bank-import/BankImportUpload.svelte`, `frontend/src/routes/(app)/bank-import/+page.svelte`, `frontend/src/lib/shared/i18n-keys.test.ts`
- `docs/manual/fr/user-manual.tex` (+ `.pdf`), `docs/manual/fr/admin-manual.tex` (+ `.pdf`)
- `docs/user-guide/fr/getting-started.md`, `docs/i18n-glossaire.md`, `docs/testing.md`, `CHANGELOG.md`
- `_bmad-output/implementation-artifacts/epic-26-choix-autonomes.md` (C-26-3-21 à 23), `sprint-status.yaml`

## Change Log

- 2026-10-10 — Fiche créée (`bmad-create-story`, autonome), 12 AC ; choix C-26-3-1 à 8.
- 2026-10-10 — **Validation P1 remédiée** (Sonnet, contexte frais, deux lentilles : R auditeur
  d'acceptation, F adversaire de périmètre complet ; prompt `26-3-validate-prompt-p1.md`, rapports
  `kesh-gate-logs/26-3-validate-p1-{R,F}.md`). **Bilan : R 2 HIGH / 5 MEDIUM / 7 LOW ; F 2 HIGH /
  5 MEDIUM / 6 LOW** ; après dédoublonnage (R-1 = HIGH-2, R-2 = HIGH-1, R-4 ≈ MEDIUM-2, R-5 =
  MEDIUM-1, R-8 ⊂ MEDIUM-3), **2 HIGH et 8 MEDIUM distincts, tous traités** ; LOW : 13 traités (dont R-8, fondu dans MEDIUM-3), 0
  écarté, 0 reclassé en dette. Toutes les entrées sont des défauts de la fiche d'origine (aucune
  remédiation antérieure) : pas de signal de recyclage.
  - **R-1 = HIGH-2** (G20 insensible à la casse frappe « son propre compte de banque ») → AC 9 ciblée,
    sensible à la casse, positive « propre compte de banque » ; AC 10 et AC 11.2 alignés (C-26-3-9).
  - **R-2 = HIGH-1** (R5/R6 rechargent la base, oracles `$C` rouges) → AC 6 réécrite : section 4-bis
    en `SAUVEGARDE_BASE=autre`, R6 rechargement supprimé, R5-bis et nouveau R6 prouvent « la garde
    passe » par le refus en aval (C-26-3-10).
  - **R-3** (aucun oracle discriminant) → AC 5.3/5.4 fixent deux libellés distincts (`image Kesh`,
    `aucun conteneur du service`) et conservent `est « <ETAT> »` ; AC 6 : motif par cas, absence des
    motifs des autres cas.
  - **R-4 ≈ MEDIUM-2** (montage et état de sortie) → AC 6 : `compose_factice(service, image, commande)`,
    fabrication de l'orphelin sans `--remove-orphans`, état de sortie prescrit et vérifié avant la
    section 5 ; mutations à un seul rouge par cas.
  - **R-5 = MEDIUM-1** (séparateur, ligne vide) → AC 5.2 : `|`, `IFS='|'`, ligne vide ignorée, ligne
    illisible = refus ; R7 (conteneur hors compose) ; AC 7.5 et mutation (e) (C-26-3-9).
  - **R-6** (« Modification d'un compte » : trois affirmations fausses) → AC 1.3 inventaire clos ; AC 10
    motifs d'absence ajoutés, valeurs d'avant relevées.
  - **R-7** (garde-fou d'étiquette inexistant) → AC 6 : étiquette construite, validée, refusée si
    préexistante, `rmi` conditionnel (C-26-3-10).
  - **MEDIUM-3 ⊃ R-8** (sites oubliés `:1756`, `:1826`, `getting-started.md:74`, `i18n-glossaire.md:151`,
    e-banking tiers) → AC 2.4 inventaire par grep (7 lignes → 0), AC 2.5, AC 4 (C-26-3-12).
  - **MEDIUM-4** (create vs up --no-start ; manuel muet) → une seule commande, AC 5.4, AC 8.4
    (C-26-3-11).
  - **MEDIUM-5** (filet hôte entier, recette) → pré-vol en tête de recette, `docs/testing.md`.
  - **LOW** : R-9 (CHANGELOG, une seule section : AC 11.1, C-26-3-8 amendé) ; R-10 (couverture :
    G20 étendue au titre et au manuel admin ; `aria-label` et `<title>` non gardés, assumés) ; R-11
    (nom du conteneur dans `docker inspect`) ; R-12 (`:1788`, `:1723`) ; R-13 (profil présélectionné,
    « Compte principal ») ; R-14 (pas de `$1` dans la boucle) ; LOW-1 (`SAUVEGARDE_RESEAU (frontend`
    retiré des absences : G16 (g) l'exige) ; LOW-2 (lecteur des quatre catalogues + anti-test-muet) ;
    LOW-3 (chaque mutation de G16 nomme sa clause) ; LOW-4 (libellés sur une ligne, pas de `|`
    suivi de `docker compose`) ; LOW-5 (« refuse de commencer », angles morts `index.docker.io`,
    miroirs) ; LOW-6 (renvoi `sec:comptes-bancaires` de la 26-2, AC 2.6).
  - Grep du symptôme après remédiation : `git grep -nIF "Compte de banque"` (2 sites de défaut + 0
    nouveau), commande de l'AC 2.4 (7 lignes, inventaire complet), `git grep -nI "Import bancaire
    CAMT.053"` (glossaire et catalogues seulement, hors spécification historique) ; dans la fiche et le
    registre, plus aucune prescription de `docker compose create`, de G20 insensible à la casse, de
    R5/R6 qui rechargent, ni de `recette.sh:31` comme garde-fou d'étiquette.
  - Nombre d'AC inchangé : **12**. Prochaine étape : validation P2 (Opus, R/F, complète),
    `26-3-validate-prompt-p2.md`.
- 2026-10-10 — **Validation P2 remédiée** (Opus, contexte frais, lentilles R et F ; prompt
  `26-3-validate-prompt-p2.md`, rapports `kesh-gate-logs/26-3-validate-p2-{R,F}.md`, relecture de
  `f5c4a958`). **Bilan : R 0 HIGH / 5 MEDIUM / 12 LOW ; F 0 HIGH / 9 MEDIUM / 8 LOW.** Après
  dédoublonnage : **9 MEDIUM distincts** (*corrigé en P3, R-L2 = F-3 : « 10 » était faux contre sa propre liste*) — R-M1 ≈ F-M2 (montage de 4-bis), R-M2 = F-M3 (mutation « garde
  d'origine »), R-M3 = F-M5 (mutations de G16), R-M4 = F-M7 (greps de l'AC 11.2), R-M5 = F-M1 (hôte
  entier), F-M4 (classes de motifs), F-M6 (profil après aperçu), F-M8 (glossaire), F-M9 (nettoyage
  hors compose) ; LOW distincts traités ou rendus sans objet (ci-dessous). **Origine** : 8 MEDIUM sur 10
  introduits par `f5c4a958` — *corrigé en P3 :* **7 introduits stricts, plus F-M9 (défaut d'origine sur un
  site introduit), plus 1 recyclé** ; R-M5 = F-M1 est un **recyclage** de P1 MEDIUM-5 → **signal D5 déclaré**.
  **Décision de l'orchestrateur** : pas de découpage, **simplification** — l'AC 6 est refondue
  (C-26-3-13, C-26-3-14).
  - **Trend** : P1 2 HIGH / 8 MEDIUM distincts (tous d'origine) → P2 0 HIGH / **9** MEDIUM distincts (7
    introduits par la remédiation, 1 d'origine sur site introduit, 1 recyclé — *corrigé en P3*). La sévérité baisse (plus de HIGH) mais se déplace vers ce
    que la remédiation a écrit, sur un seul site (la recette à vrais conteneurs) : c'est ce site qui est
    retiré.
  - **R-M5 = F-M1, R-M1 ≈ F-M2, F-M9, R-L1, R-L2, R-L4, R-L10, F-L8** (4-bis : hôte, montage, nettoyage,
    étiquette, pré-vol) → **sans objet** : la section neuve n'emploie aucun conteneur (AC 6 refondue,
    C-26-3-13) ; dépendance des sections réelles à l'hôte écrite en prérequis de `docs/testing.md`.
  - **R-M2 = F-M3** (R2 rougit aussi sous la garde d'origine) → mutations de l'AC 6 en tableau, **cas
    rouges exacts** par mutation, déroulés à la remédiation (O : 11 rouges / 6 verts ; F : 3 ; Z : 2 ;
    I : 2 ; R : 2).
  - **F-M4 = R-L3** (« motif d'aucun autre cas ») → **classes** de verdict (ZÉRO, SERVICE, FILET,
    ILLISIBLE, PASSE, ERREUR), motif de sa classe présent, motifs des autres classes absents.
  - **R-M3 = F-M5** (« elle seule » intenable) → AC 7.6 : ensemble exact des clauses rouges par mutation ;
    regex 7.1 indifférente au séparateur pour que (e) ne touche que 7.5.
  - **R-M4 = F-M7, R-L8** (greps non bornés, Change Log P1 faux) → AC 11.2 en tableau : périmètre borné
    (`':!_bmad-output'`), attendu chiffré, sites restants nommés (README:41, brochure:388, commentaires
    `errors.rs`, `bank_imports.rs:1`, `messages.ftl:1043`, spécification) ; AC 2.4 complétée. **La phrase du
    Change Log P1** « glossaire et catalogues seulement, hors spécification historique » est **fausse** :
    `git grep -nI "Import bancaire CAMT.053"` rend aussi README:41, brochure:388 et quatre commentaires de
    code — corrigée ici, laissée telle quelle plus haut pour la généalogie.
  - **F-M6** (changer le profil après l'aperçu) → AC 2.3 réécrite, vérifiée au code ; défaut d'écran :
    issue P2 à ouvrir par l'orchestrateur (C-26-3-15).
  - **F-M8** (glossaire) → colonne « précédent » = `audit-log-entity-bank-imports` (C-26-3-16).
  - **LOW traités** : R-L5 = F-L3 (G20 à `\s+`, `[Tt]outes`, mutations retour à la ligne et titre
    `en-CH`, positive `Compte comptable lié`) ; R-L6 = F-L6 (cibles DE/EN/IT fixées, C-26-3-17) ; F-L7
    (`<title>` et `aria-label` lus par i18n) ; R-L7 = F-L5 (AC 3.2 : 4 lignes, `kesh-prd-v0.2.md:252`) ;
    R-L9 (test de replis cité à tort) ; R-L11 = F-L2 (`N=$((N+1))`) ; R-L12 (registre C-26-3-5/8 ; rapport
    d'import `:1657`-`:1663` ; « QR-IBAN (optionnel) ») ; F-L1 (angle mort : étiquette déplacée après
    `pull`, fixé par G15) ; F-L4 (encadré des AC aligné sur l'AC 10.2).
  - **Écart à la consigne, consigné** : le faux `docker` « en tête de PATH » serait masqué par
    `kesh-restore.sh:54` — fonction bash exportée à la place (C-26-3-14).
  - Grep de la valeur après remédiation : dans la fiche, plus aucune prescription de 4-bis, de pré-vol,
    de `KESH_TAG`, de `compose_factice` paramétré, de « elle seule », de « motif d'aucun autre cas », de
    « valeur d'écran » ni de « traductions de même sens » (`grep -nE` sur la fiche hors Change Log :
    seul « « elle seule » est abandonné », qui le dit) ; registre : C-26-3-10 marqué remplacé,
    C-26-3-5/8/12 amendés.
  - Nombre d'AC inchangé : **12**. Prochaine étape : validation P3 (Sonnet, R/F, complète — la
    remédiation change une règle de test), `26-3-validate-prompt-p3.md`.
- 2026-10-10 — **Validation P3 remédiée** (Sonnet, contexte frais, lentilles R et F ; prompt
  `26-3-validate-prompt-p3.md`, rapports `kesh-gate-logs/26-3-validate-p3-{R,F}.md`, relecture de
  `f9d5a107`). **Bilan : R 0 HIGH / 2 MEDIUM / 9 LOW ; F 0 HIGH / 4 MEDIUM / 8 LOW.** Après dédoublonnage
  (R-M1 = F-1 + F-2, R-M2 ⊃ F-5/F-6, R-L2 = F-3, R-L4 = F-7) : **4 MEDIUM distincts** — pathspec et attendu
  de l'AC 11.2 (R-M1 = F-1/F-2), portée du faux `docker` (R-M2), décompte du Change Log P2 (F-3 = R-L2,
  MEDIUM chez F, LOW chez R), cas PASSE du scénario #619 (F-4). **Origine** : 3 introduits par `f9d5a107`,
  1 d'origine (F-4 : trou de couverture déjà présent dans 4-bis). **La simplification tient** (17 cas, 5
  mutations déroulés exacts par les deux lentilles). Pas de recyclage : les MEDIUM sont distincts de ceux de
  la P2 ; F ne lève pas de signal D5, R le déclare « à rapporter » sans recyclage — rapporté ici.
  - **Trend** : P1 2 HIGH / 8 MEDIUM → P2 0 HIGH / 9 MEDIUM → **P3 0 HIGH / 4 MEDIUM** (dont 1 de compte
    rendu). Sévérité décroissante, concentrée sur le texte écrit à la passe précédente.
  - **R-M1 = F-1/F-2** → AC 11.2 : `':(exclude)_bmad-output'` (forme courte refusée par git 2.53 : vert
    muet), code de sortie exigé (jamais 128), attendu `service=kesh-api` = `recette.sh:67` ; relevé à la
    remédiation : `toutes les banques` et `service=kesh-api` code 0, `compose create` et
    `SAUVEGARDE_SERVICE` code 1 (0 ligne, commande valide).
  - **R-M2 ⊃ F-5/F-6** → AC 6 : fonction définie **et** exportée dans le sous-shell du cas, `FAUX_*`
    exportées, `FAUX_SORTIE` toujours un fichier, `SAUVEGARDE_*` désarmées par cas, `2>&1`, `case` à motifs
    entre guillemets, assertion `type -t docker` = `file` (C-26-3-18).
  - **F-4 (+ R-L5)** → six cas neufs S18-S23 ; mutations F′, D, T ajoutées ; tous les ensembles recomptés
    sur 23 cas (O 13 rouges, F 5, F′ 2, Z 2, I 2, R 3, D 1, T 1).
  - **F-3 = R-L2** → Change Log P2 corrigé en place (9 MEDIUM distincts ; 7 + 1 + 1), marqué « corrigé en
    P3 ». Le message du commit `f9d5a107` porte encore « 10 MEDIUM » : non réécrit (historique), la fiche
    fait foi.
  - **LOW traités** : R-L1 (cas renommés S1-S23, plus de collision avec G14-G17) ; R-L3 (glossaire :
    `Import bancaire` (`audit-log-entity-bank-imports`), justification corrigée, C-26-3-16) ; R-L4 = F-7
    (`RECETTE_SECTION=0`, `ECHECS` sans interruption, environnement isolé) ; R-L6 (motifs `est « running »`
    et `est « paused »` aux lignes 153/162) ; R-L7 (C-26-3-8 aligné sur l'AC 11.1) ; R-L8 (positives en
    `[\s~]+`, `SAUVEGARDE_SERVICE` brut tenu par listing / prose par G16 (g), formes hors couverture écrites,
    « la plupart … pas toutes ») ; R-L9 (prérequis complété : dépendance neuve du filet ; passage au gate
    **signalé**, non prescrit) ; F-8 (`[\s~]+` dans G20) ; F-9 (G14 lit `up`, C-26-3-19) ; F-10 (mutation (d)
    : positive de G20 dans `textes_coherents`) ; F-11 (base MariaDB de DSM hors Docker, jointe par IP LAN :
    AC 8.1 et 8.6) ; F-12 (`<title>` au tiret long, C-26-3-20) ; P3-R (le profil reste choisi après
    Annuler, AC 2.3). Aucun reclassé en dette, aucun écarté.
  - Grep de la valeur après remédiation : dans la fiche, plus de `':!_bmad-output'` hors Change Log, plus
    de cas `G1`-`G17` hors Change Log et registre historique, plus d'attendu « 0 » pour `service=kesh-api`,
    plus de `\s+` seul entre les mots de G20 ; commande d'inventaire `up` de l'AC 7.8 rejouée (*corrigé en P4
    ciblée, L5 :* **26** lignes et non 17 ; « 17 lignes, » du relevé d'origine était faux ;
    6 nomment `kesh-api`).
  - Nombre d'AC inchangé : **12**. Prochaine étape : passe **ciblée** P4 (Haiku, une lentille) sur le seul
    commit de remédiation P3, `26-3-validate-prompt-p4-ciblee.md`.
- 2026-10-11 — **Validation P4 ciblée remédiée — validation close** (Haiku 5.5, une lentille « chasseur de
  régression de la remédiation P3 », contexte frais, sur le seul commit `e591b066` aplati ; prompt
  `26-3-validate-prompt-p4-ciblee.md`, rapport `kesh-gate-logs/26-3-validate-p4-ciblee.md`). **Bilan :
  0 HIGH / 1 MEDIUM / 5 LOW**, axes exercés et non exercés déclarés ; 23 cas et 8 ensembles de mutation
  redéroulés, conformes. Le MEDIUM a été **vérifié par l'orchestrateur** (`recette.sh:25`-`:37`).
  - **M1** (né de la remédiation P3 : `RECETTE_SECTION=0` « sans Docker », alors que `mktemp`, `openssl`
    et `trap nettoyer EXIT` — qui appelle le vrai `docker` — s'exécutent avant la section 0) → AC 6,
    « Sélecteur » : ordre de `recette.sh` imposé (trap minimale `rm -rf "$W"`, section 0 et sortie en mode
    0, puis `openssl`, `nettoyer` et sa trap), prérequis du mode 0 écrits exactement (C-26-3-21).
  - **L1** (prérequis `mktemp`/`openssl`) → fondu dans M1 : `mktemp` listé, `openssl` déplacé après.
  - **L2** (attendu conditionnel de `service=kesh-api`) → AC 11.2 : **1 exactement**, le faux compare au
    service `<S>` passé en argument, jamais au littéral.
  - **L3** (« 16 `} — Kesh` ») → AC 4 et C-26-3-20 : 16 **dans un `<title>`**, 17 lignes au grep brut.
  - **L4** (« cinq messages ») → AC 9 : six messages `fr-CH`, `:1178` ajouté.
  - **L5** (« 17 lignes ») → Change Log P3 corrigé en place : 26 lignes.
  - Signal D5 déclaré par la lentille pour M1 (né de la remédiation) : **pas de découpage** — la
    remédiation finale ne porte que sur le **texte de la fiche** et du registre, **aucune ligne de code de
    production** (la story n'est pas développée) : la boucle se clôt selon la § *La passe ciblée*.
  - **Trend de la validation** : P1 (Sonnet) 2 HIGH / 8 MEDIUM distincts → P2 (Opus) 0 HIGH / 9 MEDIUM →
    P3 (Sonnet) 0 HIGH / 4 MEDIUM → P4 ciblée (Haiku) 0 HIGH / 1 MEDIUM / 5 LOW, remédiés au texte.
    Modèles : Sonnet ↔ Opus pour les passes complètes, Haiku pour la seule passe ciblée (D6). Aucun
    finding reclassé en dette.
  - Grep de la valeur après remédiation (fiche) : `grep -nF 'ni Docker'` → seule cette entrée du Change
    Log ; `grep -nF 'cinq messages'` → seule cette entrée ; `grep -nF '17 lignes'` → l'AC 4 (« 17 lignes
    au grep brut », juste) et les entrées qui corrigent ; `grep -nF "plus, s'ils"` → 0.
  - Story passée à `in-progress`.


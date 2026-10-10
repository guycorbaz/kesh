# Story 26-3 : Le manuel décrit les écrans réels, et la garde de `kesh-restore.sh` échoue fermée

Status: ready-for-dev

<!-- Spécifiée le 2026-10-10 sur fe162504 (branche story/26-3, worktree kesh-26-3), en autonomie
     (consignes communes des agents, memory/consignes-agents-epic15.md). Choix C-26-3-1 à 8 au registre
     _bmad-output/implementation-artifacts/epic-26-choix-autonomes.md. Numéros de ligne : ceux de
     fe162504 ; les 26-1 et 26-2, développées en parallèle, touchent les mêmes fichiers (voir
     « Conflits attendus ») — le T0 relève les sites de nouveau, par les commandes, jamais de confiance.
     Validation P1 (Sonnet, lentilles R et F, sur 33002beb) remédiée : choix C-26-3-9 à 12 ; bilan au
     Change Log. Validation P2 (Opus, R et F, sur 15422ee4) remédiée : signal D5 (recyclage), AC 6
     refondue par simplification (faux docker) ; choix C-26-3-13 à 17. -->

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
   Le manuel ne décrit **jamais** « changer le profil puis confirmer ». Le défaut d'écran lui-même
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
   | `docs/i18n-glossaire.md:151` | colonne **« précédent »** : `Import bancaire CAMT.053` (valeur de `bank-import-labels-page-title`, que l'AC 4 change) | **`audit-log-entity-bank-imports`** — la clé qui atteste encore les quatre formes : `Import bancaire` / `Bankimport` / `Importazione bancaria` / `Bank import` (`messages.ftl` fr `:2266`, de `:2212`, it `:2213`, en `:2218`) ; les quatre termes de la ligne ne changent pas (validation P2, F-M8, C-26-3-16) |
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
| `<title>` en dur, `bank-import/+page.svelte:48` | « Import bancaire - Kesh » | **`{i18nMsg('bank-import-labels-page-title', …)} - Kesh`** — clé existante, forme déjà employée (`credit-notes/+page.svelte:30`, `open-items/+page.svelte:16`) |
| `docs/i18n-glossaire.md:151`, colonne « précédent » | `Import bancaire CAMT.053` | `audit-log-entity-bank-imports` (AC 2.4) |

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
   image construite localement sous un autre nom (validation P1, LOW-5). Le cas G15 de la recette le fixe.
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

### AC 6 — La garde se prouve contre un faux `docker` (#619, C-26-3-13, C-26-3-14)

> **Refondue en remédiation P2** (signal D5, recyclage P1 MEDIUM-5 → P2 M1/M5 : la garde lit l'état
> Docker de **tout l'hôte**, que la recette ne maîtrise pas). Décision de l'orchestrateur, **par
> simplification, sans découpage** (C-26-3-13) : la nouvelle section de la recette **n'emploie aucun
> vrai conteneur**. Plus de pré-vol de l'hôte, d'étiquette d'image, de `compose_factice` paramétré,
> d'`up --no-start`, de conteneur hors compose ni de nettoyage pour ces cas ; plus aucune dépendance aux
> conteneurs `kesh-dev`/`kesh-api` du poste. **Les sections existantes de la recette, réelles, restent
> inchangées.**

`scripts/synology/recette.sh` gagne une section **« 0. Garde de kesh-restore.sh contre un faux
docker »**, placée **avant** la mise en place (`recette.sh:92`) : elle ne lit ni ne crée aucun
conteneur, et rougit vite.

**Le faux `docker` est une fonction bash exportée, pas un exécutable en tête de PATH** (C-26-3-14).
`kesh-restore.sh:54` fait `export PATH="/usr/local/bin:/usr/bin:/bin:${PATH:-}"` : un faux `docker` placé
en tête du `PATH` de l'appelant serait **masqué par `/usr/bin/docker`**, et la section interrogerait le
vrai démon — le défaut même qu'on retire. Une fonction exportée (`export -f docker`) est importée par le
`bash` qui exécute le script et **précède toute recherche dans le `PATH`**. Elle n'est exportée **que dans
le sous-shell de chaque cas** (`( export -f docker; … bash "$SCRIPTS/kesh-restore.sh" … )`), jamais dans
le shell de la recette, dont les sections réelles appellent le vrai `docker`.

Comportement de la fonction (variables `FAUX_SORTIE`, `FAUX_JOURNAL`, `FAUX_CODE`, propres au cas) :
- elle **journalise** chaque appel (`printf '%s\n' "$*" >> "$FAUX_JOURNAL"`) ;
- pour les deux formes de `ps` ci-dessous, si `FAUX_CODE` ≠ 0 : écrit « Cannot connect to the Docker
  daemon » sur la sortie d'erreur et rend `FAUX_CODE` ;
- `ps --all --format {{.State}}|{{.Image}}|{{.Label "com.docker.compose.service"}}|{{.Names}}` (forme
  exacte de l'AC 5.2) : rend le contenu de `FAUX_SORTIE` tel quel ;
- `ps --all --filter label=com.docker.compose.service=<S> --format {{.State}}` (forme **d'origine**,
  `fe162504:kesh-restore.sh:76`, pour la seule preuve de mutation « garde d'origine ») : rend la
  **première colonne** des lignes de `FAUX_SORTIE` dont la **troisième** vaut `<S>` (découpage `|`) ;
- **tout autre appel** — `run`, `exec`, `compose`, `rm`, une autre forme de `ps` — : écrit
  `FAUX DOCKER : appel inattendu : <args>` sur la sortie d'erreur **et** `INATTENDU <args>` au journal,
  rend **97**. Le rechargement, la sonde et le dump de sécurité passant tous par `docker run`, **aucune
  action sur une base n'est possible** sous le faux : c'est la preuve que le script s'arrête avant.

**Montage commun**, sous `$W/garde/` (aucun fichier de `$W/kesh`) : un dossier `SAUVEGARDE_DOSSIER`
portant un `kesh-restore.cnf` factice (contenu indifférent : le script n'en teste que la présence,
`kesh-restore.sh:65`) ; un dossier source portant un dump **fabriqué** — `printf 'CREATE DATABASE
\x60kesh\x60;\n' | gzip > kesh_pre_backup.sql.gz` et son `.sha256` —, sans base ni conteneur. Chaque cas
lance `SAUVEGARDE_DOSSIER=$W/garde/dossier SAUVEGARDE_BASE=autre [SAUVEGARDE_SERVICE=…] bash
"$SCRIPTS/kesh-restore.sh" "$W/garde/source"` dans le sous-shell ci-dessus. **`SAUVEGARDE_BASE=autre`
est gardé** : quand la garde passe, le script s'arrête de lui-même à l'étape 3 sur son propre message
(« le dump recrée la base « kesh », non « autre » », `kesh-restore.sh:93`), sans même atteindre la sonde ;
l'oracle « la garde passe » est ainsi un message **du script**, non un refus du faux.

**Classes de verdict, et leur motif** (validation P2 R-L3 = F-M4 : on exige le motif de **sa** classe et
l'**absence des motifs des autres classes**, jamais « le motif d'aucun autre cas ») :

| Classe | Motif grepé (`grep -F`, sauf mention) |
|---|---|
| ZÉRO | `aucun conteneur du service « <S> »` |
| SERVICE | `(service « <S> ») est « <ETAT> »` |
| FILET | `image Kesh` |
| ILLISIBLE | `ligne de docker ps illisible` |
| PASSE | `recrée la base` |
| ERREUR | aucun des cinq motifs ; code non nul |

**Les cas** (lignes de `FAUX_SORTIE` séparées par ` ⏎ `, écrites telles quelles par `printf` ; `<S>` =
`kesh-api` sauf réglage) :

| Cas | `FAUX_SORTIE` | Réglage | Classe |
|---|---|---|---|
| G1 service renommé, image locale | `running\|kesh-local:dev\|kesh\|kesh-kesh-1` | — | ZÉRO |
| G2 idem, réglé | idem | `SAUVEGARDE_SERVICE=kesh` | SERVICE (`kesh`, `running`) |
| G3 orphelin arrêté, renommé Kesh en marche | `exited\|gcorbaz/kesh:0.12.1\|kesh-api\|kesh-api` ⏎ `running\|gcorbaz/kesh:0.13.0\|kesh\|kesh-kesh-1` | — | FILET |
| G4 réglé, orphelin `kesh-api` en marche | `exited\|gcorbaz/kesh:0.13.0\|kesh\|kesh-kesh-1` ⏎ `running\|gcorbaz/kesh:0.12.1\|kesh-api\|kesh-api` | `SAUVEGARDE_SERVICE=kesh` | FILET |
| G5 aucun conteneur | *(vide)* | — | ZÉRO, et `up --no-start` dans le message |
| G6 `created` | `created\|gcorbaz/kesh:0.13.0\|kesh-api\|kesh-api` | — | PASSE |
| G7 `restarting` | `restarting\|gcorbaz/kesh:0.13.0\|kesh-api\|kesh-api` | — | SERVICE |
| G8 `paused` | `paused\|gcorbaz/kesh:0.13.0\|kesh-api\|kesh-api` | — | SERVICE |
| G9 hors compose (étiquette vide) | `running\|mariadb:10.11\|\|kesh-recette-db` ⏎ `exited\|gcorbaz/kesh:0.13.0\|kesh-api\|kesh-api` | — | PASSE |
| G10 ligne vide intercalée | `exited\|gcorbaz/kesh:0.13.0\|kesh-api\|kesh-api` ⏎ *(vide)* ⏎ `running\|mariadb:10.11\|\|x` | — | PASSE |
| G11 ligne incomplète | `exited\|gcorbaz/kesh:0.13.0\|kesh-api\|kesh-api` ⏎ `running\|mariadb:10.11` | — | ILLISIBLE |
| G12 cinq champs | `exited\|gcorbaz/kesh:0.13.0\|kesh-api\|kesh-api` ⏎ `running\|mariadb:10.11\|a\|b\|c` | — | ILLISIBLE |
| G13 deux projets, même service | `exited\|gcorbaz/kesh:0.13.0\|kesh-api\|p1-kesh-api` ⏎ `running\|gcorbaz/kesh:0.13.0\|kesh-api\|p2-kesh-api` | — | SERVICE |
| G14 image par empreinte, autre service | `running\|docker.io/gcorbaz/kesh@sha256:<64 hex>\|autre\|x` ⏎ `exited\|gcorbaz/kesh:0.13.0\|kesh-api\|kesh-api` | — | FILET |
| G15 image affichée par identifiant (**angle mort assumé**) | `running\|407fccb51e71\|kesh\|kesh-kesh-1` ⏎ `exited\|gcorbaz/kesh:0.13.0\|kesh-api\|kesh-api` | — | PASSE |
| G16 image voisine non Kesh | `running\|gcorbaz/kesh-autre:1\|autre\|y` ⏎ `exited\|gcorbaz/kesh:0.13.0\|kesh-api\|kesh-api` | — | PASSE |
| G17 démon injoignable | *(sans objet)*, `FAUX_CODE=1` | — | ERREUR |

**Oracles de chaque cas** : code de sortie **non nul** ; motif de sa classe présent, motifs des autres
classes absents ; journal = **exactement une ligne**, qui commence par `ps --all` (forme de l'AC 5.2 ; ou forme d'origine
sous la seule mutation (O) — la forme exacte est tenue par G16, AC 7), **sans** `INATTENDU`
(anti-test-muet : un journal vide prouverait que le vrai `docker` a répondu — rouge) ;
`$W/garde/dossier/dump/.verrou` absent après le cas ; aucun `avant-restauration/` sous `$W/garde/dossier`.
G15 fixe l'angle mort au lieu de le taire : si un jour le filet lit les identifiants, ce cas change de
classe **délibérément**.

**Preuves de mutation** — chacune jouée sur la seule section 0, sortie au Dev Agent Record, script
restauré **et** touché après chaque ; **la liste des cas rouges est exacte, non « au moins »** (validation
P2 R-M2 = F-M3) :

| Mutation de `kesh-restore.sh` | Cas rouges | Cas verts |
|---|---|---|
| (O) garde d'origine (`git show fe162504:scripts/synology/kesh-restore.sh`, lignes 75-79) | G1, G2, G3, G4, G5, G7, G8, G11, G12, G13, G14 | G6, G9, G10, G15, G16, G17 |
| (F) filet retiré (seul le service est gardé) | G3, G4, G14 | les 14 autres |
| (Z) sortie sur zéro conteneur retirée | G1, G5 | les 15 autres |
| (I) contrôle « illisible » retiré | G11, G12 | les 15 autres |
| (R) `SAUVEGARDE_SERVICE` ignoré (`kesh-api` en dur) | G2, G4 | les 15 autres |

Déroulé de (O), qui sert de contrôle : la garde d'origine ignore le réglage et ne lit que l'étiquette
`kesh-api` ; elle passe sur G1, G2, G3, G5, G11, G12, G14 (motif PASSE au lieu de celui attendu) et refuse
G4, G7, G8, G13 avec son ancien libellé « un conteneur kesh-api est « … » », qui ne porte ni `(service «`
ni `image Kesh`.

**Ce que la section ne prouve pas, écrit** : le comportement du **vrai** `docker ps --format` (séparateur,
`{{.Label …}}` sur un conteneur sans étiquette) — relevé au poste à la remédiation P1
(`running|407fccb51e71|mariadb|kesh-mariadb-dev`) et rejoué par la section 4 existante, qui fait tourner la
garde nouvelle contre de vrais conteneurs (`running`, `restarting`, `paused`) ; les sections 5 à 9 la font
passer sur un `kesh-api` arrêté.

**Prérequis de la recette réelle, écrit dans `docs/testing.md`** (pas de pré-vol, décision C-26-3-13) :
les sections 4 à 9 font tourner la garde nouvelle sur **tout l'hôte** ; un conteneur **actif** d'image
`gcorbaz/kesh`, ou du service `kesh-api`, hors du projet de recette les fait refuser — le message de la
garde **nomme** le conteneur en cause, si bien que le rouge se diagnostique seul. C'était déjà vrai pour
le service `kesh-api` avec la garde d'origine.

Autres contraintes : `docs/testing.md` § *Recette des scripts de sauvegarde Synology* (`:459`-) décrit la
section 0, le faux `docker` et ses limites ; la recette se lance depuis le worktree, **hors gate**.

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
   | (d) retirer `SAUVEGARDE_SERVICE` du manuel | (g) égalité des ensembles ; (g) citation `SAUVEGARDE_SERVICE (kesh-api` (7.3) |
   | (e) remplacer `|` par une tabulation dans le `--format` et l'`IFS` | lecture à séparateur `|` (7.5) seule |

   G16 agrège ses erreurs (`echouer_si`, `configuration_transmise.rs:3466`) : copier **toutes** les
   lignes d'erreur de chaque run et constater qu'elles sont exactement celles du tableau. Restaurer et
   `touch` après chaque mutation (cargo garde sinon le binaire muté).
7. La clause existante « aucune commande `docker compose` » (`(?m)(?:^\s*|[;&|]\s*)docker compose\b`)
   doit rester verte : les libellés qui citent `docker compose up --no-start` sont des chaînes sur une
   seule ligne de code, jamais en tête de ligne ni après `;`, `&`, `|` hors guillemets — **attention**
   au `|` : le libellé de l'AC 5.4 ne doit contenir aucun `|` suivi de `docker compose` (validation P1,
   LOW-4).

G14 (`les_services_cites_existent`) reste verte : les commandes `docker compose stop|start kesh-api`
du manuel restent écrites avec le service livré.

### AC 8 — Le manuel d'administration dit le compose adapté (#619, C-26-3-4, C-26-3-7)

Dans `docs/manual/fr/admin-manual.tex` :

1. `:1671` — les réglages : ajouter `\texttt{SAUVEGARDE\_SERVICE} (\keshcommand{kesh-api})`, avec ce
   qu'il désigne (le service de Kesh dans votre compose, lu par `kesh-restore.sh` seul) ; dire de
   `SAUVEGARDE_RESEAU` qu'il désigne un réseau Docker **existant** d'où la base est joignable — en
   pratique **celui de `kesh-api`**, qui l'atteint déjà.
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
   atteignent la base » + renvoi au réglage si `kesh-api` vit sur un autre réseau.
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
(`user-manual.tex:245`, `:1166`, `:1492`, `:1580`, `:1755`, `admin-manual.tex:2472`) et dans cinq
messages du catalogue (`messages.ftl:30`, `:34`, `:35`, `:791`, `:2035` et suivants) ; une garde
insensible à la casse rougirait sur du code correct, et serait « réparée » en devenant muette.

- **négatives**, sur la source `.tex` **brute** (pas de `normaliser`, qui ne garantit pas les
  guillemets) des trois manuels français, recherche de sous-chaîne **sensible à la casse** :
  expressions régulières, `\s+` entre les mots pour qu'un **retour à la ligne de source** ne les
  contourne pas (validation P2, R-L5 = F-L3) : `\\(?:textbf|emph)\{Compte\s+de\s+banque\}`,
  `«\s*~?\s*Compte\s+de\s+banque\s*~?\s*»` (`\s` Unicode couvre U+00A0 et U+202F ; le manuel
  n'emploie ni `\og` ni `\enquote`, relevé P2), `[Tt]outes\s+les\s+banques\s+suisses` ;
- **négatives sur les catalogues** : dans **chacun des quatre** (`fr-CH`, `de-CH`, `en-CH`, `it-CH`),
  la valeur de `bank-import-labels-drop-zone` contient `CSV`, et celle de
  `bank-import-labels-page-title` ne contient pas `CAMT.053` (validation P1, R-10). `catalogue_fr()`
  (`textes_coherents.rs:98`) ne lit que `fr-CH` (LOW-2) : la garde le généralise en
  `catalogue(locale)` (même lecteur, continuation de lignes comprise) ou en écrit un second, et
  **asserte que chaque catalogue lu porte plus de 1000 clés** (anti-test-muet) ;
- **positives adossées, même fichier** : `user-manual.tex` porte `propre compte de banque` (la forme
  juste existe : la négative n'est pas muette par lecture vide) et `Compte comptable lié` — **c'est
  elle qui tient la phrase de l'AC 1.2** (absente aujourd'hui) ; `Comptes bancaires`, déjà présente
  (`:208`, `:229`, `:2577`), ne prouve que la lecture (R-L5 iii) ; `admin-manual.tex` porte `SAUVEGARDE_RESEAU=proxy` et
  `SAUVEGARDE_SERVICE` (R-10 : le paragraphe « Compose adapté » est tenu) ; chaque catalogue porte les
  deux clés ;
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
   `-- . ':!_bmad-output'` (artefacts BMAD et rétrospectives : ne se corrigent pas) :
   | Commande | Attendu après la story |
   |---|---|
   | `git grep -nIF "Compte de banque" -- docs` | 0 |
   | `git grep -nIi "toutes les banques" B` | 0 |
   | `git grep -nI "Import bancaire CAMT.053" -- crates/kesh-i18n/locales frontend/src docs/i18n-glossaire.md docs/manual/fr/user-manual.tex docs/manual/fr/admin-manual.tex` | seulement `messages.ftl:1043` (commentaire de story) |
   | `git grep -nI "Import bancaire CAMT.053" B` | les sites ci-dessus **plus**, assumés (AC 2.4) : `README.md:41`, `marketing-brochure.tex:388` (noms de fonctionnalité), `errors.rs:644`, `:2032`, `routes/bank_imports.rs:1` (commentaires de story), `docs/kesh-specifications.txt:1534`, `:2458` (historique) |
   | commande de l'AC 2.4 (`emph{Importer}`, `emph{Import bancaire}`, `Quotidien → Importer**`, `Nouvel import`, sur `docs`) | 0 |
   | `git grep -nI "compose create" B` | 0 (une seule commande, `up --no-start`) |
   | `git grep -nI "service=kesh-api" B` | 0 hors la mutation (O) **documentée** dans `docs/testing.md` s'il la cite |
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

- [ ] **T0 — Relevé** (AC tous)
  - [ ] Relancer chaque commande d'inventaire des AC sur l'état courant (`git log --oneline -1`), noter
        les écarts avec `fe162504` (les 26-1/26-2 mergées entre-temps décalent les lignes).
  - [ ] `occ` sur les PDF d'avant, valeurs ≥ 1 consignées (AC 10).
  - [ ] Créer la base du worktree et le port attribué par l'orchestrateur (consignes, points 6 et 8).
- [ ] **T1 — Manuel utilisateur** (AC 1, 2, 3)
  - [ ] § Ajout / Modification d'un compte sur l'inventaire des dialogues ; phrase « compte bancaire ».
  - [ ] § Import dans Kesh, § Import avec un profil existant ; `\label{sec:import-csv}`.
  - [ ] `:1638` reformulé (plupart, CSV, contrôle de solde à la main, MT940 non lu).
- [ ] **T2 — Écran d'import** (AC 4) : 4 catalogues, `aria-label`, titre et `<title>`, replis.
- [ ] **T3 — `kesh-restore.sh`** (AC 5) : réglage, lecture unique, filet image, sortie fermée, en-tête.
- [ ] **T4 — Recette** (AC 6) : section 0 contre le faux `docker` (fonction exportée), cas G1-G17,
      cinq mutations (O, F, Z, I, R) aux cas rouges exacts, `docs/testing.md` (section 0, prérequis
      de la recette réelle).
- [ ] **T5 — G16** (AC 7) et **G20** (AC 9), avec leurs mutations.
- [ ] **T6 — Manuel d'administration** (AC 8).
- [ ] **T7 — PDF** (AC 10), **CHANGELOG** et propagation (AC 11).
- [ ] **T8 — Gates** (AC 12), commit par étape BMAD, messages `refs #616 #617 #619`.

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

### Debug Log References

### Completion Notes List

- Spécification : analyse de contexte complète (issues, epic, scripts, gardes G14/G16/G19, écrans,
  catalogues, manuels) — fiche prête pour la validation.

### File List

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
  dédoublonnage : **10 MEDIUM distincts** — R-M1 ≈ F-M2 (montage de 4-bis), R-M2 = F-M3 (mutation « garde
  d'origine »), R-M3 = F-M5 (mutations de G16), R-M4 = F-M7 (greps de l'AC 11.2), R-M5 = F-M1 (hôte
  entier), F-M4 (classes de motifs), F-M6 (profil après aperçu), F-M8 (glossaire), F-M9 (nettoyage
  hors compose) ; LOW distincts traités ou rendus sans objet (ci-dessous). **Origine** : 8 MEDIUM sur 10
  introduits par `f5c4a958` ; R-M5 = F-M1 est un **recyclage** de P1 MEDIUM-5 → **signal D5 déclaré**.
  **Décision de l'orchestrateur** : pas de découpage, **simplification** — l'AC 6 est refondue
  (C-26-3-13, C-26-3-14).
  - **Trend** : P1 2 HIGH / 8 MEDIUM distincts (tous d'origine) → P2 0 HIGH / 10 MEDIUM distincts (8
    introduits par la remédiation, 1 recyclé). La sévérité baisse (plus de HIGH) mais se déplace vers ce
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


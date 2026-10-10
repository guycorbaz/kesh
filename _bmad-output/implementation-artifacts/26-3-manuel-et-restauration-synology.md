# Story 26-3 : Le manuel décrit les écrans réels, et la garde de `kesh-restore.sh` échoue fermée

Status: ready-for-dev

<!-- Spécifiée le 2026-10-10 sur fe162504 (branche story/26-3, worktree kesh-26-3), en autonomie
     (consignes communes des agents, memory/consignes-agents-epic15.md). Choix C-26-3-1 à 8 au registre
     _bmad-output/implementation-artifacts/epic-26-choix-autonomes.md. Numéros de ligne : ceux de
     fe162504 ; les 26-1 et 26-2, développées en parallèle, touchent les mêmes fichiers (voir
     « Conflits attendus ») — le T0 relève les sites de nouveau, par les commandes, jamais de confiance. -->

## Story

En tant qu'utilisateur qui prépare son premier import de relevé, et qu'exploitant d'un NAS Synology
dont le compose n'est pas exactement celui qui est livré,
je veux que le manuel me décrive les écrans et les formats tels qu'ils sont, et que le script de
restauration refuse de recharger tant qu'il ne sait pas que Kesh est arrêté,
afin de ne pas chercher une case qui n'existe pas, ni un export que ma banque n'offre pas, et de ne
jamais recharger la base sous un Kesh en marche.

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
> `SAUVEGARDE_RESEAU (frontend` → **1**, `quel que soit le nom du projet` → **1**,
> `SAUVEGARDE_SERVICE` → **0** (admin). Et pour le contrôle visuel, la forme aplatie demandée par le
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
   bancaires* (*Nouveau compte bancaire* : Banque, IBAN, QR-IBAN facultatif) et on le lie par
   **« Compte comptable lié »** (ou, pour un compte bancaire existant, le bouton **« Lier au plan
   comptable »**). Seuls y sont proposés les comptes **actifs, postables, de type Actif ou Passif**,
   hors compte débiteurs et compte créanciers désignés (`bank-accounts/+page.svelte:105`-`:115`). Renvoi
   vers ce que le manuel dit déjà de ce lien (`user-manual.tex:240`-`:272`, solde comptable).
3. § *Modification d'un compte* (`:348`-`:350`) dit ce que fait l'écran : numéro jamais modifiable,
   compte parent **non modifiable** par l'écran (à vérifier contre la route de mise à jour : si l'API
   l'accepte mais que l'écran ne l'offre pas, le manuel décrit l'écran).
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
3. **Hors périmètre** : § *Profils banque* (`:1669`-`:1681`, création d'un profil). La Story 26-1 y
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
   Sur `fe162504` : **3** lignes — `user-manual.tex:1638` (le site), `user-manual.tex:817` (« exigé
   par les banques suisses sur les QR-factures », juste) et `website/about.html:116` (« format expected
   by Swiss banks », pain.001, juste). Brochure, README, site : aucune promesse d'universalité
   (`git grep -niI camt -- website README.md docs/manual/fr/marketing-brochure.tex` relu ligne à ligne).

[#618]: https://github.com/guycorbaz/kesh/issues/618

### AC 4 — L'écran d'import ne dit plus « CAMT.053 » seul (#617, C-26-3-2)

Sites, **inventaire clos** relevé par `git grep -n "bank-import-labels-drop-zone\|bank-import-labels-page-title" -- frontend/src crates/kesh-i18n` et `grep -n 'aria-label="Glisser' frontend/src/lib/features/bank-import/BankImportUpload.svelte` :

| Site | Aujourd'hui | Cible (`fr-CH`) |
|---|---|---|
| `bank-import-labels-drop-zone`, 4 catalogues (`fr-CH:1077`, `de-CH`/`en-CH`/`it-CH:1018`) | « Glissez votre fichier CAMT.053 ici… » | « Glissez votre fichier CAMT.053 ou CSV ici ou cliquez pour parcourir » (= repli actuel, `BankImportUpload.svelte:258`) |
| `aria-label` en dur, `BankImportUpload.svelte:232` | « …un fichier CAMT.053 ou cliquer… » | « Glisser-déposer un fichier CAMT.053 ou CSV, ou cliquer pour parcourir » |
| `bank-import-labels-page-title`, 4 catalogues (`fr-CH:1075`, autres `:1016`) + repli `bank-import/+page.svelte:52` | « Import bancaire CAMT.053 » | « Import de relevés bancaires » |
| `<title>` en dur, `bank-import/+page.svelte:48` | « Import bancaire - Kesh » | aligné sur le titre de page |

DE/EN/IT : traductions de même sens (pas de « CAMT.053 » seul). Repli Svelte = valeur `fr-CH`
(`i18n-un-repli-par-cle.test.ts`). Aucun test ne dépend de ces textes (E2E par `data-testid`,
`bank-import-page-title`, relevé : `grep -rn "Import bancaire\|Glissez" frontend/tests frontend/src --include=*.ts` → 0).

### AC 5 — La garde de `kesh-restore.sh` échoue fermée (#619, C-26-3-3)

Dans `scripts/synology/kesh-restore.sh`, étape 1 (`:75`-`:79`), **toujours avant le verrou** :

1. Nouveau réglage `SAUVEGARDE_SERVICE=${SAUVEGARDE_SERVICE:-kesh-api}`, déclaré avec les autres
   (`:60`-`:63`).
2. Une seule lecture de l'hôte, en **affectation nue** (une erreur de `docker` vaut refus, `set -e`) :
   `docker ps --all --format` portant l'état, l'image, l'étiquette `com.docker.compose.service`
   (`{{.Label "com.docker.compose.service"}}`) et le nom du conteneur.
3. Sont **gardés** : les conteneurs dont le service vaut `$SAUVEGARDE_SERVICE`, **et** ceux dont
   l'image est `gcorbaz/kesh` — `^(docker\.io/)?gcorbaz/kesh([:@].*)?$` —, quel que soit leur service.
   Tout conteneur gardé dans un autre état que `exited|created|dead` → refus, `exit 1`, message qui
   nomme le conteneur, son service, son image et son état, et la commande d'arrêt avec **son**
   service. La forme `case "$ETAT" in exited|created|dead) ;; *) … exit 1;; esac` est conservée (liste
   fermée, C-15-14-81).
4. **Aucun** conteneur du service `$SAUVEGARDE_SERVICE` sur l'hôte → refus, `exit 1`, « rien n'a été
   touché », message qui dit : si le service de Kesh porte un autre nom dans votre compose, posez
   `SAUVEGARDE_SERVICE=<nom>` ; si Kesh n'est pas encore installé sur cet hôte, créez son conteneur
   sans le démarrer (`docker compose create <service>`).
5. L'en-tête du script (`:5`-`:10`, `:20`, `:49`-`:51`) et le message final (`:127`, « docker compose
   start kesh-api ») disent la garde telle qu'elle est et emploient `$SAUVEGARDE_SERVICE`.
6. `kesh-dump.sh` n'a pas de garde et ne lit pas `SAUVEGARDE_SERVICE` ; il n'est pas modifié, sauf son
   en-tête si une phrase y décrit la garde (vérifier).

### AC 6 — La recette rejoue le service renommé (#619)

`scripts/synology/recette.sh` (Story 15-14b) gagne une section, **après** la section 4 et avant la 5,
qui ne laisse rien derrière elle (orphelins retirés : `docker compose up -d --remove-orphans`), et dont
chaque cas vérifie la base inchangée (`empreinte` = état courant) et aucun dump de sécurité :

| Cas | Montage (compose factice du projet `kesh-recette-synology`) | Attendu |
|---|---|---|
| R1 | service renommé `kesh`, en marche ; aucun conteneur `kesh-api` | sans réglage : **refus**, message qui nomme `SAUVEGARDE_SERVICE` |
| R2 | idem | `SAUVEGARDE_SERVICE=kesh` : **refus** « est « running » » |
| R3 | `kesh` arrêté ; **orphelin `kesh-api` en marche** sur l'image étiquetée `gcorbaz/kesh:recette-synology-<aléa>` | `SAUVEGARDE_SERVICE=kesh` : **refus** par le filet de l'image (l'orphelin n'est pas du service réglé) |
| R4 | orphelin `kesh-api` **arrêté** + `kesh` **en marche** sur la même image étiquetée | sans réglage : **refus** par le filet de l'image (le service par défaut, arrêté, satisferait la seule garde fermée) |
| R5 | aucun conteneur du service | **refus** ; puis `docker compose create kesh-api` → la garde passe (état `created`) |
| R6 | `kesh` arrêté, `SAUVEGARDE_SERVICE=kesh` | la garde passe ; le rechargement aboutit (code 0) et la base = dump restauré |

Contraintes :

- l'étiquette d'image de R4 est **propre à la recette** (aléa), créée par `docker tag "$IMG" …`, et
  retirée par `nettoyer()` (`docker rmi` de l'**étiquette** seule — jamais une image `gcorbaz/kesh`
  préexistante) ; le garde-fou de tête (`recette.sh:31`) l'interdit si elle ne contient pas
  `recette-synology` ;
- la section 4 existante (Kesh `running`, `restarting`, `paused`, sur le service `kesh-api`) reste
  verte et inchangée dans son sens ;
- `compose_factice` reçoit le nom du service en paramètre (aujourd'hui `kesh-api` en dur,
  `recette.sh:68`-`:71`) ; `etat_kesh_api` et `kesh_api` restent sur `kesh-api` ;
- **preuve de mutation** : la nouvelle section est jouée contre la garde d'origine (copie de
  `git show fe162504:scripts/synology/kesh-restore.sh`), et doit rendre **RECETTE ROUGE** sur R1 et R4
  au moins ; puis le script est restauré **et touché** (`touch`) — sortie des deux runs au Dev Agent
  Record ;
- `docs/testing.md` § *Recette des scripts de sauvegarde Synology* (`:459`-) liste les nouveaux cas ;
- la recette se lance depuis le worktree, **hors gate** (≈ 2 min), ne touche à aucun autre conteneur
  — jamais `kesh-mariadb-dev`.

Les conteneurs factices tournent sur `mariadb:10.11` : **seuls** R3 et R4 portent l'image étiquetée
Kesh, sans quoi le filet par l'image ne serait pas exercé (et R1, R2, R5, R6 ne doivent pas la porter,
sans quoi leur refus viendrait du filet et non du réglage). Preuve de mutation de R3 et R4 : retirer le
filet seul → ces deux cas rougissent, les autres non.

### AC 7 — G16 lit la garde nouvelle (#619, C-26-3-5)

`crates/kesh-api/tests/configuration_transmise.rs`, `synology_sauvegarde_la_base_par_le_dump` :

1. la regex de l'étape « refus Kesh actif » (`:3121`-`:3124`, qui exige aujourd'hui
   `--filter label=com.docker.compose.service=kesh-api`) suit la forme nouvelle : `docker ps --all`
   avec `{{.State}}`, `{{.Image}}` et l'étiquette de service ; l'**ordre** des étapes reste contrôlé ;
2. clauses neuves : défaut de `SAUVEGARDE_SERVICE` = un service de `docker-compose.prod.yml`
   (`services_prod`) ; sortie sur zéro conteneur du service (`exit 1` sous la condition) ; filet par
   l'image `gcorbaz/kesh` présent ;
3. (g) : l'égalité des ensembles de réglages manuel/scripts inclut `SAUVEGARDE_SERVICE` ; le manuel le
   cite sous la forme `SAUVEGARDE_SERVICE (kesh-api` suivie d'une virgule ou d'une parenthèse ; la
   boucle « mêmes défauts dans les deux scripts » **n'inclut pas** ce réglage (le dump ne le lit pas) —
   l'écrire en commentaire ;
4. la doc-comment de G16 (`:2866`-`:2934`) et ses angles morts sont mis à jour (C-26-3-3, angle mort
   de l'image construite localement) ;
5. **preuve de mutation**, chacune rejouée et consignée : (a) remettre la garde d'origine → G16 rouge ;
   (b) retirer la sortie sur zéro conteneur → G16 rouge ; (c) retirer le filet par l'image → G16
   rouge ; (d) retirer `SAUVEGARDE_SERVICE` du manuel → G16 rouge. Restaurer et `touch` après chaque
   mutation (cargo garde sinon le binaire muté).

G14 (`les_services_cites_existent`) reste verte : les commandes `docker compose stop|start kesh-api`
du manuel restent écrites avec le service livré.

### AC 8 — Le manuel d'administration dit le compose adapté (#619, C-26-3-4, C-26-3-7)

Dans `docs/manual/fr/admin-manual.tex` :

1. `:1671` — les réglages : ajouter `\texttt{SAUVEGARDE\_SERVICE} (\keshcommand{kesh-api})`, avec ce
   qu'il désigne (le service de Kesh dans votre compose, lu par `kesh-restore.sh` seul) ; dire de
   `SAUVEGARDE_RESEAU` qu'il désigne un réseau Docker **existant** d'où la base est joignable — en
   pratique **celui de `kesh-api`**, qui l'atteint déjà.
2. Un paragraphe **« Compose adapté »** (reverse proxy, service renommé) : réseau et service suivent le
   compose réellement déployé ; commande qui affiche les réseaux du conteneur
   (`docker inspect -f '{{range $n, $_ := .NetworkSettings.Networks}}{{$n}} {{end}}' kesh-api`, à
   vérifier à l'exécution) ; rappel qu'un réseau **déclaré dans le compose** porte le préfixe du projet
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
4. *Recovery depuis Snapshot*, étapes d'arrêt et de redémarrage (`:1724`-`:1738`) : « le service de
   Kesh (`kesh-api` dans le compose livré) » ; la commande reste écrite avec `kesh-api` (G14, G16 (f)).
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

- aucune occurrence de « Compte de banque » (insensible à la casse, après `normaliser`) dans les trois
  manuels français ; aucune de « toutes les banques suisses » ; aucune valeur de
  `bank-import-labels-drop-zone` sans « CSV » dans les quatre catalogues ;
- **chaque négative adossée à une positive sur le même fichier** : le manuel utilisateur porte
  « Comptes bancaires » et « Compte comptable lié » ; chaque catalogue porte la clé
  `bank-import-labels-drop-zone` ;
- preuve de mutation : réintroduire la phrase → rouge ; restaurer, `touch`.

### AC 10 — PDF régénérés et contrôlés (C-26-3-6)

1. `scripts/mem-guard.sh make -B -C docs/manual admin user` ; PDF commités avec leurs sources ; la
   brochure n'est pas régénérée (non touchée).
2. Par `occ` (en tête des AC), PDF d'avant ≥ 1 puis PDF neuf = **0** : `Compte de banque`,
   `fourni par toutes les banques suisses`, `Nouvel import CSV`, `quel que soit le nom du projet` (si
   la phrase est réécrite), `SAUVEGARDE_RESEAU (frontend)` sous son ancienne forme si elle change ;
   PDF neuf ≥ 1 : `Compte comptable lié`, `Confirmer l'import`, `SAUVEGARDE_SERVICE`,
   `SAUVEGARDE_RESEAU=proxy`, et la phrase neuve sur le CAMT.053 (motif choisi au dev). Toutes les
   valeurs au Dev Agent Record, commande comprise.
3. Aucun `??` de référence non résolue (`\ref{sec:import-csv}`) : `pdftotext … | grep -c '??'` sur les
   deux PDF, avant/après.

### AC 11 — CHANGELOG et propagation (C-26-3-8)

1. `CHANGELOG.md` : section `## [0.13.1] — Non publié` (créée si absente, au-dessus de `[0.13.0]`,
   forme exacte que lit `scripts/prepare-release.sh:237`), entrées sous `### Corrigé` pour #616, #617,
   #619, et une ligne **« Action requise »** pour l'exploitant Synology : re-télécharger
   `kesh-restore.sh` au tag de la version, poser `SAUVEGARDE_SERVICE` si son service ne s'appelle pas
   `kesh-api`, `SAUVEGARDE_RESEAU` s'il n'est pas sur `frontend`.
2. Grep du symptôme **par valeur** après chaque patch, sur tout le dépôt suivi :
   `git grep -nI "Compte de banque"`, `git grep -nIi "toutes les banques"`,
   `git grep -nI "service=kesh-api"`, `git grep -nI "SAUVEGARDE_SERVICE"` (sites attendus : script,
   manuel, G16, recette, testing.md, CHANGELOG — liste au Dev Agent Record). Les artefacts BMAD
   (`_bmad-output/`) et les rétrospectives ne se corrigent pas.
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
- [ ] **T4 — Recette** (AC 6) : section R1-R6, nettoyage de l'étiquette, preuve de mutation,
      `docs/testing.md`.
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
  `(?m)(?:^\s*|[;&|]\s*)docker compose\b` — les **messages** peuvent citer `docker compose create`,
  jamais une ligne de commande), `$1` employé une seule fois (`SOURCE=$(cd "$1" && pwd -P)`).
- **Piège `set -e` et `while read`** : une boucle alimentée par `<<< "$LISTE"` tourne dans le shell
  courant (un `exit 1` y sort bien du script) ; un `docker ps … | while read` tournerait dans un
  sous-shell, où `exit 1` ne sortirait que du sous-shell — **garde ouverte**, exactement le défaut à
  corriger. Le compteur de conteneurs du service doit lui aussi survivre à la boucle.
- **`{{.Image}}`** affiche l'image **telle que référencée à la création** (`gcorbaz/kesh:latest`,
  `gcorbaz/kesh:0.13.0`, éventuellement `docker.io/gcorbaz/kesh:…`) — ou un identifiant si l'étiquette
  a été retirée : d'où l'angle mort écrit (C-26-3-3).
- **G16 est un couplage écrit** : sa doc-comment le dit (« une écriture équivalente rougirait »). La
  modifier fait partie de la story, pas d'un écart.
- **La recette** réécrit le compose factice à chaque `compose_factice` avec `--force-recreate` ; un
  changement de **nom** de service laisse l'ancien conteneur orphelin — c'est le cas R3/R4 qu'on veut,
  et c'est aussi ce qu'il faut retirer (`--remove-orphans`) entre les cas qui ne le veulent pas.

### Conflits attendus avec les Stories 26-1 et 26-2 (développées en parallèle)

| Fichier | 26-1 | 26-2 | 26-3 |
|---|---|---|---|
| `docs/manual/fr/user-manual.tex` | § Profils banque, import CSV | affichage des comptes bancaires | § Ajout d'un compte, § Import dans Kesh, `:1638`, § Import avec un profil existant |
| `BankImportUpload.svelte` | profil CSV ? | `{account.bankName} ({account.iban})` `:212` | `aria-label` `:232` |
| catalogues `messages.ftl` | clés du profil | clés de la description | `bank-import-labels-drop-zone`, `-page-title` |
| `CHANGELOG.md` | `[0.13.1]` | `[0.13.1]` | `[0.13.1]` |
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

Pas de bibliothèque neuve. `docker ps --format` avec `{{.Label "…"}}` et `docker compose create`
vérifiés présents sur le poste (Docker Compose 2.40.3) ; sur DSM, Container Manager embarque Compose
v2 — `create` y existe depuis la v2.0, mais le manuel ne peut l'affirmer pour toutes les versions de
DSM : écrire « à défaut, `docker compose up --no-start <service>` ».

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

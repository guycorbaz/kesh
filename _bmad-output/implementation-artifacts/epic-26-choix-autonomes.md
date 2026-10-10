# Epic 26 — registre des choix faits en autonomie

Registre des choix tranchés sans question, par story, au moment où ils sont faits. Chaque entrée dit
le contexte, l'option retenue, les options écartées et la réversibilité. **Mise à jour par union**
(main + branches), jamais par écrasement ; un numéro en double se renumérote, il ne s'écrase pas.

---

## C-26-3-1 — 26-3 (spec) : #616 traité comme « le formulaire du compte », non comme une ligne

- **Contexte** : #616 signale la case « Compte de banque » (`user-manual.tex:345`, `:1652`). La liste
  où elle figure (`§ Ajout d'un compte`) décrit d'autres champs que le formulaire n'a pas non plus :
  « Devise », les types « Bilan / Résultat », le libellé « Libellé » (le formulaire dit « Nom ») ; le
  paragraphe « Modification d'un compte » dit le compte parent modifiable et le numéro modifiable
  tant qu'aucune écriture n'est saisie, alors que le dialogue d'édition n'a pas de champ parent et
  désactive toujours le numéro. Même symptôme (le manuel décrit un écran qui n'existe pas), même liste.
- **Retenu** : inventaire **clos** des champs des deux dialogues (`accounts/+page.svelte`, création
  `:532`-`:590`, édition `:618`-`:660` sur `fe162504`) ; la liste du manuel est réécrite sur cet
  inventaire, et dit comment un compte devient compte bancaire (Administration → Comptes bancaires).
  Les deux procédures d'import (`Import dans Kesh`, `Import avec un profil existant`) sont réécrites
  sur l'écran réel (`/bank-import` : « Importer des relevés », prévisualisation, « Confirmer l'import »).
- **Écarté** : corriger la seule ligne citée (laisse trois champs fictifs à côté) ; réécrire aussi
  « Profils banque » (création d'un profil) — la Story 26-1 modifie les réglages du profil et réécrit
  cette sous-section : la toucher ici créerait un conflit certain, sans gain.
- **Réversible** : oui (texte).

## C-26-3-2 — 26-3 (spec) : #617, les libellés d'écran « CAMT.053 seul » corrigés avec le manuel

- **Contexte** : le grep du symptôme (« le CAMT.053 est la voie ») trouve, hors du manuel, la zone de
  dépôt de l'import : `bank-import-labels-drop-zone` vaut « Glissez votre fichier CAMT.053 ici… » dans
  les **quatre** catalogues, alors que l'écran accepte `.csv` (`accept=".xml,…,.csv,text/csv,.txt"`) et
  que le repli Svelte dit déjà « CAMT.053 ou CSV » ; l'`aria-label` en dur et le titre de page
  (`bank-import-labels-page-title` = « Import bancaire CAMT.053 ») disent de même.
- **Retenu** : les trois textes disent « CAMT.053 ou CSV » (titre : « Import de relevés bancaires »,
  aligné sur le menu « Importer des relevés »), dans les quatre locales, replis compris.
- **Écarté** : laisser l'écran à une story ultérieure — c'est exactement l'écart que #617 reproche au
  manuel, sur l'écran que l'utilisateur a devant lui.
- **Réversible** : oui. Coût : la story touche du code d'écran, donc E2E complet au dernier commit.

## C-26-3-3 — 26-3 (spec) : #619, une garde FERMÉE, réglable, avec un filet par l'image

- **Contexte** : la garde de `kesh-restore.sh` (`:76`) ne voit que les conteneurs étiquetés
  `com.docker.compose.service=kesh-api` ; un service renommé n'est pas vu, la boucle ne tourne pas, le
  rechargement part sous un Kesh en marche. Un renommage dans un compose laisse de plus l'ancien
  conteneur **orphelin** (Compose ne le retire qu'avec `--remove-orphans`) : un `kesh-api` arrêté
  peut coexister avec un `kesh` en marche.
- **Retenu**, trois mécanismes cumulés :
  1. **réglage `SAUVEGARDE_SERVICE`** (défaut `kesh-api`, un service de `docker-compose.prod.yml`),
     comme les quatre `SAUVEGARDE_*` existants ;
  2. **fermée** : si **aucun** conteneur de ce service n'existe sur l'hôte, refus (le script ne sait
     pas ce qu'il garde) — message qui nomme `SAUVEGARDE_SERVICE` et, pour un hôte où Kesh n'est pas
     encore installé, la commande qui crée le conteneur sans le démarrer (`docker compose create
     <service>`, état `created`, admis — **amendé par C-26-3-11** : `docker compose up --no-start`) ;
  3. **filet** : est aussi refusé tout conteneur actif dont l'image est `gcorbaz/kesh` (toute
     étiquette ou empreinte, préfixe `docker.io/` admis), quel que soit son service — c'est lui qui
     voit l'orphelin et le service renommé.
- **Écarté** : (a) réglage seul, garde ouverte sur zéro conteneur — un oubli du réglage redonne le
  défaut d'origine ; (b) garde fermée seule — l'orphelin `kesh-api` arrêté la satisfait pendant que
  `kesh` tourne ; (c) image seule — une image construite localement ou une étiquette déplacée
  (`docker ps` affiche alors un identifiant) passe ; (d) auto-détection du service par le compose —
  le script ne lit pas le compose et ne doit pas en dépendre (C-15-14-77) ; (e) option `--force` —
  la garde n'a pas d'issue de contournement, c'est sa raison d'être.
- **Angle mort écrit** : une image Kesh construite sous un autre nom, dans un service autre que
  `SAUVEGARDE_SERVICE`, n'est vue par rien — elle exige le réglage. Deux installations Kesh sur le
  même hôte se gênent davantage (refus, sens sûr ; déjà écrit en C-15-14-77).
- **Réversible** : oui.

## C-26-3-4 — 26-3 (spec) : #619, le réseau du dump reste un réglage explicite, documenté

- **Contexte** : `SAUVEGARDE_RESEAU` (défaut `frontend`) doit désigner un réseau Docker **existant**
  d'où la base est joignable. Le manuel n'en montre aucun exemple, et sa propre recette Traefik
  (`§ sec:traefik`) rattache `kesh-api` à `kesh-internal`, réseau créé par Compose sous le nom
  `<projet>_kesh-internal` : avec elle, le défaut `frontend` échoue.
- **Retenu** : le manuel dit la règle (le réseau de `kesh-api`, qui atteint déjà la base), donne la
  commande qui l'affiche (`docker inspect` sur le conteneur), et l'exemple d'un reverse proxy
  (`SAUVEGARDE_RESEAU=proxy`), pour la tâche planifiée comme pour le rechargement. Le listing de
  suppression d'une base illisible garde `--network frontend` (la recette le réécrit par `sed`) ; la
  prose dit de le remplacer par la valeur de `SAUVEGARDE_RESEAU`.
- **Écarté** : déduire le réseau du conteneur `kesh-api` dans le script — un second réglage implicite,
  et une dépendance au nom de conteneur ; changer le listing en `--network <réseau>` — casse la recette
  (`recette.sh:280`) pour un gain de forme.
- **Réversible** : oui.

## C-26-3-5 — 26-3 (spec) : gardes — G16 amendée, une garde de texte neuve

- **Retenu** : G16 (`crates/kesh-api/tests/configuration_transmise.rs`) lit la nouvelle forme de la
  garde (sa regex exige aujourd'hui `--filter label=com.docker.compose.service=kesh-api`), le réglage
  `SAUVEGARDE_SERVICE` (défaut = service de `docker-compose.prod.yml`, cité au manuel sous la forme
  `SAUVEGARDE_SERVICE (kesh-api`), la sortie sur zéro conteneur et le filet par l'image. Une garde de
  texte neuve, **G20** dans `textes_coherents.rs`, interdit « Compte de banque » (**forme du défaut,
  sensible à la casse — amendé par C-26-3-9**) et « toutes les
  banques suisses » dans les manuels français (les catalogues : valeurs de la zone de dépôt et du titre
  d'import — précisé en validation P2, R-L12), chaque négative adossée à une
  positive sur le même fichier.
- **Écarté** : ne compter que sur la recette — elle n'est pas dans le gate ; seule G16 l'est.
- ⚠️ **Numéro à vérifier au merge** : les Stories 26-1 et 26-2, spécifiées en parallèle, peuvent
  prendre G20 ; renuméroter la dernière mergée.
- **Réversible** : oui.

## C-26-3-6 — 26-3 (spec) : PDF régénérés — les deux manuels touchés seulement

- **Retenu** : `make -B -C docs/manual admin user` ; la brochure n'est pas touchée (aucune occurrence
  du symptôme, grep relevé à l'AC 3) et n'est pas régénérée, pour ne pas commiter un PDF qui ne
  diffère que par ses horodatages.
- **Écarté** : `make fr` (régénère la brochure sans motif). Les manuels DE/EN/IT n'existent pas
  (`docs/manual/{de,en,it}/` ne portent qu'un `README.md`).
- **Réversible** : oui.

## C-26-3-7 — 26-3 (spec) : un paragraphe voisin contradictoire corrigé au passage

- **Contexte** : sous le listing de téléchargement des scripts (`admin-manual.tex:1669`), le
  paragraphe dit « \keshpath{main} désigne la version courante du dépôt… remplacez \keshpath{main}
  par son étiquette », alors que le listing pointe déjà sur `v0.13.0` (G19 l'y oblige depuis la
  release 0.13.0). Même section que celle que la story réécrit, même nature (le manuel décrit autre
  chose que ce qu'il montre).
- **Retenu** : la phrase dit que l'URL désigne l'étiquette de la version installée, à garder égale à
  l'image du compose. Le numéro de version du listing **ne bouge pas** dans cette story : c'est la
  release v0.13.1 qui le portera (point 4-bis, G19).
- **Réversible** : oui.

## C-26-3-8 — 26-3 (spec) : CHANGELOG sous `## [0.13.1] — Non publié`

- **Retenu** : la story crée la section si elle n'existe pas (les 26-1 et 26-2 y écrivent aussi ;
  fusion par union au merge), entrées sous `### Corrigé`, plus une ligne « Action requise » pour
  l'exploitant Synology : re-télécharger `kesh-restore.sh` au tag de la version (`v0.13.1`), poser
  `SAUVEGARDE_SERVICE` si son service ne s'appelle pas `kesh-api`, `SAUVEGARDE_RESEAU` s'il n'est pas sur
  `frontend`, recréer le conteneur par `docker compose up --no-start` s'il a été supprimé (aligné sur
  l'AC 11.1 en validation P3, R-L7).
- **Amendé en validation P1 (R-9)** : au rebase, les entrées des trois stories se fusionnent par
  union sous **une seule** section `[0.13.1]` (`prepare-release.sh` refuse plus d'une section) ;
  contrôle `grep -c '^## \[0\.13\.1\] — ' CHANGELOG.md` = 1.
- **Réversible** : oui.

## C-26-3-9 — 26-3 (validation P1, R-1 = HIGH-2, R-5 = MEDIUM-1) : G20 ciblée ; lecture de `docker ps` à séparateur `|`

- **Contexte** : G20 prescrite « insensible à la casse » sur « Compte de banque » aurait rougi sur six
  phrases justes des manuels (« son propre compte de banque »). La lecture unique de `docker ps`, sans
  séparateur fixé, décalait les champs sur une étiquette de service vide (conteneur hors compose).
- **Retenu** : G20 cherche la **forme du défaut**, sensible à la casse, sur la source brute
  (`\textbf{Compte de banque}`, `« Compte de banque »`), avec « propre compte de banque » en positive ;
  elle lit les **quatre** catalogues (drop-zone avec « CSV », titre sans « CAMT.053 ») avec un
  anti-test-muet (> 1000 clés). Lecture `docker ps --format '{{.State}}|{{.Image}}|{{.Label …}}|{{.Names}}'`,
  `IFS='|' read -r ETAT IMAGE SERVICE NOM RESTE`, ligne vide ignorée, ligne incomplète ou à cinq champs
  = refus.
- **Écarté** : G20 limitée aux deux numéros de ligne (fragile) ; tabulation comme séparateur (IFS
  blanc : champs vides avalés).
- **Réversible** : oui.

## C-26-3-10 — 26-3 (validation P1, R-2 = HIGH-1, R-4, R-7, MEDIUM-5) : la recette ne recharge jamais dans 4-bis

> ⛔ **Remplacé par C-26-3-13** (validation P2) : la section 4-bis à vrais conteneurs est abandonnée.

- **Contexte** : R5/R6 laissaient le script recharger ; les oracles `$C` des sections 5, 5-bis, 7
  rougissaient, et une mutation faisait rougir tous les cas suivants. Le garde-fou d'étiquette cité
  (`recette.sh:31`) n'existait pas ; un Kesh étranger sur l'hôte aurait fait rougir la recette sans cause.
- **Retenu** : section « 4-bis » après la 4, chaque cas en `SAUVEGARDE_BASE=autre` — la garde qui passe
  s'arrête à l'étape 3 sur « recrée la base », sans écriture ; oracle de motif distinct par cas ; R6
  « rechargement abouti » supprimé (couvert par 6-9) ; R7 conteneur hors compose ; état de sortie
  prescrit et vérifié ; étiquette `gcorbaz/kesh:recette-synology-<8 hex>` construite, refusée si
  préexistante, retirée seule ; pré-vol « aucun `gcorbaz/kesh` actif ».
- **Écarté** : rejouer R6 après la section 9 (rechargement déjà couvert, et état final de la recette à
  remonter) ; restaurer C après chaque cas (oracle de plus, sans gain).
- **Réversible** : oui.

## C-26-3-11 — 26-3 (validation P1, MEDIUM-4) : une seule commande de recréation, `docker compose up --no-start`

- **Contexte** : la fiche donnait `docker compose create` au message et `up --no-start` en note
  technique ; le manuel ne disait pas qu'un `docker compose down` rend désormais le rechargement
  impossible sans recréer le conteneur.
- **Retenu** : `docker compose up --no-start <service>` partout (script, manuel — étape d'arrêt de la
  *Recovery* et *Stratégie 3-2-1* —, recette), avec l'angle mort : la commande lit le compose entier,
  `.env` doit être en place.
- **Écarté** : `create` (moins répandue dans les versions de Compose v2) ; les deux formes (deux
  consignes pour un geste).
- **Réversible** : oui.

## C-26-3-12 — 26-3 (validation P1, MEDIUM-3, R-8) : chemins d'écran et e-banking tiers

- **Retenu** : l'inventaire des renvois au chemin de l'écran d'import est fermé par un grep
  (`emph{Importer}`, `emph{Import bancaire}`, `Quotidien → Importer**`, `Nouvel import` : 7 lignes, 0
  attendu) — `user-manual.tex:1756`, `:1826`, `docs/user-guide/fr/getting-started.md:74` ajoutés —, et
  `docs/i18n-glossaire.md:151` suit la valeur d'écran (**amendé par C-26-3-16** : c'est la colonne
  « précédent » qui change, vers `audit-log-entity-bank-imports`). La sous-section « Récupération du fichier
  CAMT.053 » perd ses chemins de menus d'e-banking tiers (Multibanking/Klippa, PostFinance Yellownet),
  invérifiables, au profit d'une consigne générale.
- **Écarté** : garder les chemins tiers « à titre indicatif » — c'est la forme exacte de #617.
- **Hors inventaire, assumés** : `\section{Import bancaire}` (titre de chapitre), noms de fonctionnalité
  (manuel admin, brochure, README), type d'entité du journal d'audit, `docs/kesh-specifications.txt`.
- **Réversible** : oui.

## C-26-3-13 — 26-3 (validation P2, signal D5) : la garde se prouve contre un faux `docker`, sans conteneur

- **Contexte** : P2 (Opus, R 5M/12L, F 9M/8L) — presque tout né de la remédiation P1 (`f5c4a958`) et
  porté par la section 4-bis à vrais conteneurs ; **recyclage** : P1 MEDIUM-5 revient en P2 M1/M5 — la
  garde lit l'état Docker de **tout l'hôte**, que la recette ne maîtrise pas (`kesh-dev`, `kesh-api`
  d'autres compose du dépôt). Signal D5 déclaré.
- **Retenu** (décision de l'orchestrateur, **sans découpage, par simplification**) : la section neuve
  (« 0 », avant la mise en place) n'emploie **aucun** vrai conteneur. Un faux `docker` rend des sorties
  `docker ps --format` fabriquées (cas renommés **S1-S23** et complétés par C-26-3-18 ; à l'origine G1-G17 : service renommé, orphelin arrêté, `created`, `restarting`,
  `paused`, hors compose, ligne vide, ligne incomplète, cinq champs, deux projets, aucun conteneur, image
  Kesh sous un autre service, image par identifiant, image voisine, démon injoignable), journalise ses
  appels et **refuse** (code 97) toute autre sous-commande : aucune action sur une base n'est possible.
  `SAUVEGARDE_BASE=autre` reste, pour que « la garde passe » soit un message du script. Mutations aux
  cas rouges **exacts** (O, F, Z, I, R).
- **Écarté** : corriger 4-bis (pré-vol élargi, nom de service propre à la recette, `ecrire_compose`,
  `arreter_service`, nettoyage du hors-compose) — chaque correctif ajoutait une dépendance à l'hôte ;
  découper la story.
- **Conséquences** : plus de pré-vol, d'étiquette d'image, de `compose_factice` paramétré, d'`up
  --no-start` ni de nettoyage pour ces cas ; les sections réelles 1-9 inchangées ; leur dépendance à
  l'hôte (déjà vraie pour `kesh-api` avec la garde d'origine) est écrite en **prérequis** dans
  `docs/testing.md` — le message de la garde nomme le conteneur en cause.
- **Réversible** : oui.

## C-26-3-14 — 26-3 (remédiation P2) : le faux `docker` est une fonction exportée, pas un exécutable du PATH

- **Contexte** : la consigne « faux `docker` en tête de PATH » ne marche pas : `kesh-restore.sh:54`
  préfixe `PATH` par `/usr/local/bin:/usr/bin:/bin`, si bien que `/usr/bin/docker` passerait devant.
- **Retenu** : `docker() { … }; export -f docker` dans le **sous-shell de chaque cas** ; bash importe la
  fonction et la cherche **avant** le `PATH`. Anti-test-muet : chaque cas exige une ligne au journal du
  faux (journal vide = le vrai `docker` a répondu = rouge).
- **Écarté** : modifier `kesh-restore.sh` pour ne plus préfixer `PATH` (le préfixe protège la tâche
  planifiée DSM, au `PATH` minimal) ; un faux à chemin absolu (le script ne l'appellerait pas).
- **Réversible** : oui.

## C-26-3-15 — 26-3 (validation P2, F-M6) : la procédure d'import ne fait jamais changer le profil après l'aperçu

- **Contexte** : vérifié au code — l'aperçu est calculé au dépôt avec le profil alors choisi ; changer de
  profil ensuite ne le recalcule pas, et « Confirmer l'import » envoie le nouveau profil avec l'aperçu de
  l'ancien (`BankImportUpload.svelte:106`-`:116`, `:222`, `:73`-`:82`).
- **Retenu** : le manuel dit de choisir le profil **avant** de déposer ; sinon **Annuler**, choisir,
  **redéposer** (`reset()` garde le profil). Le défaut d'écran n'est **pas** corrigé ici (hors périmètre
  documentaire de la story) : **issue P2 à ouvrir par l'orchestrateur** (« l'aperçu d'import n'est pas
  recalculé quand on change de profil après le dépôt ; la confirmation importe avec un mapping non
  prévisualisé »).
- **Écarté** : corriger l'écran dans la 26-3 (change le comportement, demande ses propres tests) ;
  décrire « changer le profil puis confirmer » (chemin trompeur).
- **Réversible** : oui.

## C-26-3-16 — 26-3 (validation P2, F-M8) : glossaire — la colonne « précédent » suit une clé qui atteste encore

- **Retenu** : `docs/i18n-glossaire.md:151`, colonne « précédent » : `Import bancaire CAMT.053` →
  `audit-log-entity-bank-imports` (Import bancaire / Bankimport / Importazione bancaria / Bank import) ;
  les quatre termes ne changent pas. **Justification corrigée en validation P3 (R-L3)** : l'en-tête du
  glossaire dit que la colonne nomme la clé, mais ce bloc de lignes y porte des **valeurs d'écran**
  (`Carnet d'adresses`, `Réconciliation`…). On écrit donc, dans le style des voisines, la valeur **et** la
  clé : `Import bancaire` (`audit-log-entity-bank-imports`).
- **Écarté** : y écrire la nouvelle valeur du titre (« Import de relevés bancaires »), qui n'atteste pas
  « import bancaire » — une attestation fausse dans le document qui sert à n'en pas avoir.
- **Réversible** : oui.

## C-26-3-17 — 26-3 (validation P2, L6/L7) : cibles DE/EN/IT fixées ; titre et `aria-label` lus par i18n

- **Retenu** : sur le terme attesté `relevé (bancaire)` (glossaire `:105`, figé par `nav-bank-import`) —
  titre : `Import von Kontoauszügen` / `Bank statement import` / `Importazione di estratti conto` ; zone :
  `CAMT.053- oder CSV-Datei hier ablegen oder klicken zum Durchsuchen` / `Drop your CAMT.053 or CSV file
  here or click to browse` / `Trascinare il file CAMT.053 o CSV qui o fare clic per sfogliare`. Le
  `<title>` lit `bank-import-labels-page-title` et l'`aria-label` lit `bank-import-labels-drop-zone` (clés
  existantes, coût nul, tenus par G20).
- **Écarté** : « traductions de même sens » (à deviner) ; garder les deux textes en dur « assumés ».
- **Réversible** : oui.

## C-26-3-18 — 26-3 (validation P3, R-M2 = F-5/F-6, R-L1, R-L4 = F-7, F-4, R-L5) : le faux `docker` précisé, six cas neufs

- **Contexte** : P3 (Sonnet, R 2M/9L, F 4M/8L) — la simplification tient (17 cas, 5 mutations exacts),
  mais la fiche ne disait ni **où** la fonction est définie (définie au niveau du script, elle masquerait
  le vrai `docker` pour la mise en place et la `trap nettoyer`, muette), ni que `FAUX_*` doivent être
  exportées (sinon « unbound variable » sous `set -u`), ni comment jouer la section 0 seule ; aucun cas
  PASSE n'exerçait le scénario #619 réussi (service renommé, réglé, arrêté) ; les noms `G14`-`G17` des cas
  heurtaient ceux de gardes existantes.
- **Retenu** : fonction **définie et exportée dans le sous-shell de chaque cas**, `FAUX_*` exportées dans
  ce sous-shell, `FAUX_SORTIE` toujours un fichier (vide pour S5), `SAUVEGARDE_*` désarmées puis posées par
  cas, `2>&1` ; assertion `type -t docker` = `file` après la section 0 ; `RECETTE_SECTION=0` joue la
  section 0 seule (ni Docker ni `sqlx` — **amendé par C-26-3-21** : ordre de `recette.sh` imposé pour que ce soit vrai) ; cas renommés **S1-S23**, dont six neufs : S18 (#619 réussi), S19
  (+ orphelin arrêté), S20 (image Kesh d'un autre service à l'arrêt), S21 (`dead`, image nue), S22 (#619
  seul, en marche → FILET avant la sortie sur zéro), S23 (image nue d'un autre service → FILET) ; huit
  mutations (O, F, F′, Z, I, R, D, T) aux cas rouges recomptés sur 23 cas ; motifs `est « running »` et
  `est « paused »` ajoutés aux lignes 153 et 162 de la section 4 réelle (R-L6), seule retouche des sections
  réelles.
- **Écarté** : faire du faux un exécutable (masqué par `kesh-restore.sh:54`, C-26-3-14) ; porter la section
  0 dans le gate par un test Rust (R-L9) — non requis par les issues, **signalé au Project Lead**.
- **Réversible** : oui.

## C-26-3-19 — 26-3 (validation P3, F-9) : G14 lit `up`

- **Contexte** : `docker compose up --no-start kesh-api` devient la voie unique de recréation ; G14 ne lit
  pas `up` (verbes `exec|stop|start|restart|logs|pull|rm`).
- **Retenu** : `up` ajouté aux verbes de G14, options longues admises ; inventaire préalable (6 lignes
  nomment `kesh-api`, toutes hors sections Synology : vertes) ; mutation `up --no-start kesh` en section
  Synology → rouge.
- **Écarté** : une exigence textuelle dans G16 (f) seule — elle tiendrait la présence de la commande, pas
  l'existence du service nommé.
- **Réversible** : oui.

## C-26-3-20 — 26-3 (validation P3, F-12) : `<title>` au tiret long

- **Retenu** : `{i18nMsg('bank-import-labels-page-title', …)} — Kesh` — majorité des titres lus par clé
  (16 `} — Kesh` **dans un `<title>`** — 17 lignes au grep brut, dont une hors `<title>` — contre 8 `} - Kesh`
  sous `frontend/src/routes`, relevé P3, précisé en P4 ciblée, L3).
- **Écarté** : garder le tiret court d'origine (minorité, et sans la lecture par clé).
- **Réversible** : oui.

## C-26-3-21 — 26-3 (validation P4 ciblée, M1) : la section 0 tourne avant `openssl` et la trap qui appelle `docker`

- **Contexte** : P4 ciblée (Haiku) — `recette.sh:25`-`:37` exécute `mktemp`, `openssl` et pose
  `trap nettoyer EXIT` avant toute section ; `nettoyer` appelle le **vrai** `docker` (`rm -f`, `network rm`,
  `run mariadb:10.11`, image tirée si absente), erreurs vers `/dev/null`. « `RECETTE_SECTION=0` sans
  Docker » était vrai en résultat, faux en exécution. Vérifié par l'orchestrateur.
- **Retenu** : ordre imposé — `mktemp` et une trap **minimale** `rm -rf "$W"`, puis la section 0 (sortie
  en mode 0), puis `openssl`, garde `kesh-mariadb-dev`, `nettoyer` et `trap nettoyer EXIT`. Prérequis du
  mode 0 écrits exactement (bash, coreutils, gzip, grep, awk ; ni docker, ni openssl, ni sqlx, ni python3).
- **Écarté** : écrire le prérequis « tel quel » (docker appelé à la sortie) — la section 0 perdrait la
  propriété qui la justifie (aucune dépendance à l'hôte) ; conditionner la trap par
  `[ "${RECETTE_SECTION:-}" = 0 ] || trap …` — laisse `openssl` avant la section 0 et duplique le test.
- **Réversible** : oui.

## C-26-3-22 — 26-3 (dev) : la boucle lit la ligne entière, puis la découpe ; trois séparateurs exactement

- **Contexte** : l'AC 5.2 prescrit `while IFS='|' read -r ETAT IMAGE SERVICE NOM RESTE`. Mesuré au
  poste : sous cette forme, une ligne `|||` (quatre champs vides) est indiscernable d'une ligne vide
  — elle serait ignorée au lieu d'être refusée —, et une ligne `a|b|c|d|` (séparateur final) laisse
  `RESTE` vide : elle passerait pour lisible.
- **Retenu** : `while IFS= read -r LIGNE` (ligne vide → ignorée), puis
  `IFS='|' read -r ETAT IMAGE SERVICE NOM RESTE <<< "$LIGNE"` et un contrôle de **trois `|`
  exactement** (`SEPARATEURS=${LIGNE//[^|]/}`), en plus des champs `ETAT`, `IMAGE`, `NOM` non vides
  et de `RESTE` vide. Toujours dans le shell courant (`done <<< "$LISTE"`). La clause de G16 (AC 7.5)
  lit le `--format` à `|` et `IFS='|' read -r ETAT IMAGE SERVICE NOM RESTE` — elle tient cette forme.
- **Écarté** : la forme littérale de l'AC 5.2 (les deux trous ci-dessus).
- **Réversible** : oui.

## C-26-3-23 — 26-3 (dev) : renvois du manuel utilisateur par `\label` neufs ; la 26-2 non mergée

- **Contexte** : AC 1.2, 2.2 et 2.6 — la 26-2 (`sec:comptes-bancaires`) n'est pas mergée au
  développement (`main` = `9b724027`).
- **Retenu** : renvois au texte (*Administration → Comptes bancaires*) et `\label` neufs, posés sur
  des titres existants : `sec:ajout-compte`, `sec:solde-comptable`, `sec:import-dans-kesh`,
  `sec:import-csv` (AC 3.1), `sec:import-csv-profil`. À la fusion avec la 26-2, les renvois de
  l'AC 1.2 pourront porter `\S\ref{sec:comptes-bancaires}` — signalé à l'orchestrateur.
- **Écarté** : citer `sec:comptes-bancaires` avant qu'il existe (`??` au PDF).
- **Réversible** : oui.

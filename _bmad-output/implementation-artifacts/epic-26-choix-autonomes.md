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
     <service>`, état `created`, admis) ;
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
  texte neuve, **G20** dans `textes_coherents.rs`, interdit « Compte de banque » et « toutes les
  banques suisses » dans les manuels français et les catalogues, chaque négative adossée à une
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
  l'exploitant Synology : re-télécharger `kesh-restore.sh` au tag `v0.13.1`, poser
  `SAUVEGARDE_SERVICE` si son service ne s'appelle pas `kesh-api`.
- **Réversible** : oui.

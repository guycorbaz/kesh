# Lot C de la Story 49-5, reporté — #1499 : deux copies du test de recette prennent les mêmes ports

*Fichier écrit le 2026-11-24 (Opus 5.5), à côté de la fiche `49-5-solde-dette-epic.md`, quand le lot C en a été **retiré** par
décision de l'orchestrateur (signal de recyclage de la règle de splitting préventif, `CLAUDE.md` § *Règle de splitting préventif* :
le MEDIUM V3-1 de la passe ciblée P3 naît du correctif de V2-2, lui-même né du correctif de V1-1/V1-7, à sévérité égale). **#1499
reste ouverte**, à traiter dans une story à elle (Epic 50 suivant ou liste d'attente). Ce fichier **n'est pas une fiche** : c'est
la conception acquise par trois passes de validation, pour qu'une future fiche reparte de là. Tout y est lu à `59fb28712` ; une
future fiche relit chaque `chemin:ligne` à sa propre base.*

Rapports des passes : `/home/gcorbaz/devel/kesh-gate-logs/e49solde/validation-p1.md` (V1-1, V1-2, V1-5 à V1-7, V1-10),
`validation-p2.md` (V2-1 à V2-5, V2-8), `validation-p3-ciblee.md` (V3-1 à V3-4). Inventaire : `inv-1499.txt`. Fragilité voisine,
hors de #1499 : issue **I-1** (les quatre `D=$(grep -oE 'dossier .*$' …)`, texte dans la fiche 49-5, § *Issues à ouvrir par
l'orchestrateur*).

## Ce qui se lit au code (`59fb28712`)


1. **Le choix des ports** — `scripts/tests/recette-montee.test.sh:385-395` :
   ```bash
   # Deux ports libres, trouvés au moment du test, sous la plage éphémère du système : les
   # connexions à la base des autres tests y prendraient sinon le port entre deux contrôles.
   ports_a_l_ecoute() { ss -ltn | awk '{print $4}' | sed 's/.*://'; }
   port_libre() {
     local p=$1 pris
     pris=$(ports_a_l_ecoute)
     while /usr/bin/grep -qx "$p" <<<"$pris"; do p=$((p + 2)); done
     echo "$p"
   }
   PA=$(port_libre 23310)
   PB=$(port_libre 23410)
   ```
   « Libre » veut dire « personne n'écoute à l'instant » : **rien ne réserve** le port. Deux copies lancées ensemble lisent `ss`
   avant que l'une ou l'autre n'écoute, et prennent **les mêmes** `PA=23310`, `PB=23410`. Constat de l'issue **confirmé** au code.
2. **Ce qui fait alors collision** : le test écoute réellement sur `PORT_BASE + 1` pendant 30 s (`:864-873`, crochet de la phase 2 :
   `s.bind((\"127.0.0.1\", int(sys.argv[1]))); s.listen(1); time.sleep(30)" "$((PORT_BASE + 1))"`), et le script sous test refuse
   un port déjà pris (`scripts/recette-montee.sh:153-157` : `refus "port $p déjà pris"`) et relève à la fin un port de la recette
   resté à l'écoute (`:308-310`). Une copie voit donc le port de l'autre ; la collision se joue dans la fenêtre de 30 s de l'écoute
   du crochet. **Le mécanisme de l'interruption « sans résumé » est établi** par la fiche 49-1d (`49-1d-types-de-lignes-recette-montee.md:944`,
   « onze journaux tronqués, `mut/juge-tirage-parallele.log` », et `:1496-1502`, qui rattache ce défaut (ii) à #1499) : le script
   sous test refuse **avant** d'écrire « dossier », puis la ligne `D=$(/usr/bin/grep -oE 'dossier .*$' "$FX/sortie" | tail -1 | sed …)`
   rend 1 sous `pipefail`, et `set -e` (`:11`) arrête le test avant son résumé. Quatre sites ont cette forme : `:589`, `:755`,
   `:858`, `:875` (`grep -nF "grep -oE 'dossier .*\$'"`).
   **Périmètre** : cette story supprime la **collision de ports** (la cause constatée). La **fragilité** des quatre sites
   `D=$(grep …)` sous `set -e`/`pipefail` — toute autre cause de refus arrête encore le test sans dire pourquoi — **reste hors
   périmètre** (plafond de trois issues) : elle va à une **issue distincte, I-1**, que l'orchestrateur ouvrira avant le
   développement ; **#1499 ne couvre que la collision de ports**. La scission se fait **avant** le développement, à l'ouverture
   d'I-1 : le nouveau corps de #1499 (texte prêt, § *Issues à ouvrir par l'orchestrateur*) renvoie la fragilité à I-1.
3. **Les autres ressources partagées** (`inv-1499.txt`) : la racine jetable est unique (`mktemp -d /tmp/recette-montee-test.XXXXXX`,
   `:19`) ; `ecouter` (`:473-485`) laisse le système choisir son port (`bind(("127.0.0.1",0))`) ; aucun autre chemin fixe sous
   `/tmp` n'est écrit (la ligne `:574` est une assertion sur un chemin **dans le conteneur factice**). Les ports `23310`/`23410`
   n'apparaissent nulle part ailleurs dans `scripts/tests` ni `crates/*/tests/*`. Ensemble clos : **les ports seuls**.
4. **L'enveloppe** — `crates/kesh-db/tests/recette_montee_script.rs` lance le test sous `timeout -k 10 600` et exige le résumé
   « aucune assertion en échec ». Le test prend **337 à 389 s** (`docs/testing.md:643`, mesures du 2026-11-24) : une copie qui
   **attendrait** la fin de l'autre dépasserait les 600 s (D-C1, option écartée).
5. **La documentation** — `docs/testing.md:643` porte : « ⚠️ Ses scénarios choisissent leurs ports au moment du test : deux copies
   lancées en même temps se gênent, et l'une s'interrompt avant son résumé — une campagne de mutations les joue une à une. » Seul
   site qui décrit le symptôme (`git grep -n -iE 'deux copies|en même temps se g|mêmes ports' -- docs scripts CLAUDE.md`).
6. **Où la collision se produit réellement** : **pas** entre deux gates — `test-fast.sh`, `gate-ci.sh` et `cargo nextest` sont
   sérialisés par le verrou de voie `.gate.lock` (`docs/agents-orchestration.md` § 5.1, voie « Base et Rust »). Elle se produit dans
   une **campagne de mutations** qui lance plusieurs copies (fiche 49-1d `:944` : onze journaux tronqués) et dans tout lancement
   **hors verrou** (`bash scripts/tests/recette-montee.test.sh` direct, à côté d'un gate). La story reste justifiée (P3 ouverte,
   campagnes de mutations à venir), sur ce motif-là.


## Décisions acquises (D-C1 à D-C4)

- **D-C1 — Réserver le port, non seulement le constater, dans le shell principal** (*remédiation de V1-1, HIGH, et V1-7*).
  1. **`port_libre` ne s'appelle plus en substitution de commande.** `PA=$(port_libre 23310)` s'exécute dans un **sous-shell** : un
     descripteur ouvert là se ferme à sa sortie et le verrou tombe avant que `PA` ne serve. La fonction s'appelle **directement**
     et rend son résultat dans une **variable globale** : `port_libre 23310; PA=$PORT_RESERVE` puis `port_libre 23410 2;
     PB=$PORT_RESERVE`. **Sites d'appel réécrits** : `scripts/tests/recette-montee.test.sh:394` (`PA=$(port_libre 23310)`) et `:395`
     (`PB=$(port_libre 23410)`) — les deux seuls (`git grep -n 'port_libre' -- scripts` : `:388`, `:394`, `:395`). Les descripteurs
     sont gardés dans un tableau global (`VERROUS_PORTS+=("$fd")`), jamais fermés avant la fin du processus.
  2. **Le verrou** : pour chaque candidat `p` (et `p + 1` quand la fonction reçoit une largeur de 2, cas de `PB`), ouvrir
     `exec {fd}>"$DOSSIER_VERROUS/port-$p.lock"` (où `DOSSIER_VERROUS=${KESH_RECETTE_TEST_VERROUS:-/tmp/kesh-recette-montee-test-$(id -u)}`,
     relu à chaque appel, point 4) puis prendre `flock -n "$fd"`. **Sous `set -euo pipefail` (`:11`), `flock -n` se
     garde toujours par `if`** : `if flock -n "$fd"; then …; else exec {fd}>&-; p=$((p + 2)); continue; fi` — un `flock -n` nu en
     échec interromprait le test sans résumé, le symptôme même de l'issue. **Largeur 2** : si `p` est pris mais que `p + 1` est
     refusé (verrou tenu ou port à l'écoute), le verrou de `p` **se relâche** (`exec {fd}>&-` sur le descripteur de `p`) avant de
     passer au candidat suivant : aucun verrou orphelin ne reste tenu. L'ouverture du fichier se garde de même
     (`if ! exec {fd}>…; then …`, ou un test d'écriture préalable) et, en échec, arrête le test **avec un message** (`echo … >&2;
     exit 2`), jamais en silence.
  3. **`ss` contrôle `p` et `p + 1`** : un processus étranger à l'écoute sur `PB + 1` ferait refuser le script sous test
     (`recette-montee.sh:153-157`) ; le candidat est sauté si l'un des deux écoute.
  4. **Dossier des verrous** : `port_libre` lit la variable **`KESH_RECETTE_TEST_VERROUS`** ; son défaut est le dossier par
     utilisateur `/tmp/kesh-recette-montee-test-$(id -u)` (créé par `mkdir -p`, mode 700). C'est ce défaut que partagent les copies
     pour réserver `PA` et `PB` ; T5 le remplace par des dossiers propres à la copie (AC-C2). **Garantie, dite exactement** : un
     fichier de verrou créé par un autre compte dans un dossier partagé ne peut plus faire échouer l'ouverture, puisque chaque compte
     a son dossier ; **sauf dossier squatté** — un autre compte qui aurait créé `/tmp/kesh-recette-montee-test-<uid>` avant le premier
     lancement (`mkdir -p` réussit alors sans en changer le propriétaire) : l'ouverture échoue et le test s'arrête **avec un
     message** (point 2), cas théorique sur la station. Les fichiers de verrou **ne se
     retirent pas** (retirer un fichier dont le verrou vient d'être relâché rouvrirait la course entre deux copies) ; c'est écrit en
     commentaire au-dessus de `port_libre`.
  5. Deux copies simultanées prennent ainsi des ports distincts, sans attendre ni refuser. **T6 et une campagne de mutations à
     plusieurs copies restent verts** : `PA` et `PB` se réservent dans le dossier partagé (c'est ce que T6 éprouve), et T5 ne touche
     **jamais** ce dossier ni les plages 23310/23410 — il travaille dans des dossiers sous `$WORK` (unique par copie,
     `mktemp -d`, `:19`), sur des ports que rien n'écoute, et ses attentes sont relatives au port rendu : deux copies jouent donc T5
     en même temps sans se voir. Vérifié par la validation P1 : `PA`
     (pas de 2 depuis 23310) et `PB + 1` (impair, depuis 23411) ne se chevauchent pas avant 50 sauts.
  *Écartées* : un verrou global `flock -n` qui refuse la seconde copie (il fait rougir un lancement parce qu'un autre tourne — le
  défaut même) ; un verrou global bloquant (la seconde copie attendrait 337 à 389 s et dépasserait l'enveloppe de 600 s, § lot C,
  point 4) ; des ports choisis par le système (`bind` sur 0) puis relâchés (course entre la libération et l'usage, et `PB + 1` doit
  suivre `PB`) ; garder `$(…)` et ouvrir le verrou après coup dans le shell principal (une fenêtre de course entre le choix et le
  verrou).
- **D-C2 — Ce que la story vérifie avant d'écrire** (étape 0) : relu par la validation P1 sans objection — `exec 3>&1`/`3>&-`
  (`recette-montee.sh:317`, `:409`, `:423`) ne touchent que le descripteur 3, et la relance `env -i … bash` (`:86-97`, `exec` à `:96`) ne ferme pas
  les descripteurs ≥ 10. L'agent de développement le relit à HEAD. Les processus de fond du test héritent du descripteur : un
  verrou peut survivre quelques secondes à la sortie (écoutes de 30 et 60 s) ; la copie suivante saute alors au port d'après —
  **sans effet sur le verdict**, à écrire en commentaire au-dessus de `port_libre`. Si la relecture contredit D-C1, l'agent
  **s'arrête et le signale** au lieu d'improviser une autre mécanique.
- **D-C3 — `docs/ci.md:101`** (*remédiation de V1-10*) : ajouter `flock` (`util-linux`, présent sur `ubuntu-latest` et à la station)
  à la liste des outils requis (`python3`, `ss`, `jq`, `realpath`, `md5sum`, `timeout`), **et corriger dans le même patch** la
  valeur périmée de la même ligne — « `recette-montee.test.sh` : 99 à 104 s sous nextest » et « ajoutent une à deux minutes à la
  suite » — sur les mesures de `docs/testing.md:643` (337 à 389 s ; la phrase dit de quelle date et quel lancement). Greper les
  **valeurs** (`99 à 104`, `une à deux minutes`) sur tout le dépôt et traiter chaque site.
- **D-C4 — La phrase de `docs/testing.md:643`** se réécrit : deux copies simultanées prennent des ports distincts (verrou par port,
  dans le shell principal) ; une campagne de mutations peut donc lancer plusieurs copies **sans collision de ports** ; un refus du
  script sous test arrête encore le test avant son résumé (I-1, ouverte à part). Ne rien affirmer des gates, qui étaient déjà
  sérialisés par `.gate.lock` (§ lot C, point 6).


## Critères d'acceptation acquis (AC-C1 à AC-C4)

- **AC-C1** — `port_libre` est appelé directement aux deux sites (`:394`, `:395` d'origine) et rend `PORT_RESERVE` ; il réserve le
  port par un verrou exclusif non bloquant pris **dans le shell principal**, gardé par `if` ; `PB + 1` est réservé et contrôlé à `ss`
  avec `PB` ; dossier des verrous par utilisateur (D-C1). Constat : `git grep -n '\$(port_libre' -- scripts` **vide**, et lecture.
- **AC-C2** — **T5** (dans le test shell lui-même, section « ports ») se joue **entièrement dans des dossiers de verrous propres à la
  copie** — `KESH_RECETTE_TEST_VERROUS="$WORK/verrous-t5-<cas>" port_libre …` (affectation préfixée, qui ne vaut que pour l'appel)
  — sur des ports **distincts par cas**, hors de 23310/23410, et **ses attentes sont relatives au port rendu** (jamais une valeur
  absolue que l'état de la station pourrait déplacer) :
  - (i) **verrou tenu au retour** — après `port_libre 24310` (dossier `verrous-t5-i`), un processus de fond tente
    `flock -n <fichier du port rendu> true` — qui **rouvre** le fichier, jamais sur un descripteur hérité, dont la description
    partagée prendrait le verrou sans conflit — et **échoue** : le verrou est encore tenu par le shell principal (c'est ce cas qui
    rougit si le verrou est pris dans un sous-shell, M5) ;
  - (ii) **port tenu sauté** — un tenant de fond prend le verrou du port `q` (premier port libre à `ss` à partir de 24330, dossier
    `verrous-t5-ii`) ; `port_libre 24330` dans ce dossier rend un port **différent de `q` et supérieur** (M4) ;
  - (iii) **témoin positif** — dans un dossier `verrous-t5-iii` vide, `flock -n` sur un port jamais réservé (24390) réussit depuis
    un processus de fond ;
  - (iv) **la variable est lue** — un tenant de fond tient `r` (premier port libre à `ss` à partir de 24350) dans `verrous-t5-iv-autre` ;
    `KESH_RECETTE_TEST_VERROUS="$WORK/verrous-t5-iv-autre" port_libre 24350` rend un port ≠ `r`, et
    `KESH_RECETTE_TEST_VERROUS="$WORK/verrous-t5-iv" port_libre 24350` rend `r` (M6).
  **Tenants de fond synchronisés** (V2-2) : chacun se lance par `flock -n <fichier> bash -c 'touch "<prêt>"; sleep 60'`
  (toujours `-n`, jamais bloquant), sorties vers `/dev/null` et descripteurs du test fermés (`3>&-`, et ceux de `VERROUS_PORTS`),
  pour ne pas retenir la lecture de l'enveloppe Rust ; le test attend le fichier `<prêt>` par une boucle **bornée** (5 s, puis
  `fail` avec message), et **vérifie** par une sonde `flock -n … true` qui échoue que le verrou est bien tenu, avant d'appeler
  `port_libre`. Chaque tenant est tué et attendu (`kill`, `wait`) en fin de bloc, et un **`trap … EXIT`** étendu (`kill` des PID
  notés dans `T5_PIDS`, puis `rm -rf "$WORK"`, `:20`) garantit le nettoyage même si le test s'arrête. Témoins positif et négatif dans
  le même bloc.
- **AC-C3** — **T6** : **une seule invocation**, prise comme la voie « Base et Rust » (`docs/agents-orchestration.md` § 5.1,
  `:197` : recettes de montée et mutations), lance les deux copies en même temps :
  `flock /tmp/station-heavy.lock flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock scripts/mem-guard.sh bash -c 'bash scripts/tests/recette-montee.test.sh >
  …/t6-copie1.log 2>&1 & bash scripts/tests/recette-montee.test.sh > …/t6-copie2.log 2>&1 & wait'` (deux copies dans **une**
  place de `mem-guard`, qui n'en offre que deux sur la station : deux invocations séparées pourraient s'attendre et ne pas se
  recouvrir). Chaque copie **imprime au démarrage** `PA`, `PB` et un horodatage, et à la fin un horodatage ; le journal
  `kesh-gate-logs/e49solde/t6-deux-copies.log` réunit les deux sorties et **prouve le recouvrement** (intervalles qui se
  chevauchent) ; chaque copie rend « ✅ aucune assertion en échec » et le code 0, **T5 compris** (ses lignes ✓ figurent dans les deux journaux) ;
  les `PA`/`PB` sont distincts. Durée de chaque
  copie notée (deux copies concurrentes allongent les 337 à 389 s mesurés ; lancé directement, le test n'a pas l'enveloppe de
  600 s).
- **AC-C4** — `docs/testing.md:643` et `docs/ci.md:101` sont à jour (D-C3, D-C4) ; `git grep -n -iE 'deux copies lancées en même
  temps se gênent|99 à 104 s|une à deux minutes à la suite'` vide (ou chaque site restant justifié au Dev Agent Record).


## Tests et mutations

| Test | Lot | Où | Rouge prouvé par |
|---|---|---|---|
| **T5** — réservation (AC-C2, cas i à iv) | C | `scripts/tests/recette-montee.test.sh` | **rouge par mutations** M4, M5 et M6 |
| **M4** — mutation : `port_libre` sans verrou (retour au code de `59fb28712`) | C | idem | T5 (ii) rouge (le port tenu n'est pas sauté) ; journal `m4.log` |
| **M6** — mutation : `port_libre` ignore `KESH_RECETTE_TEST_VERROUS` (dossier par utilisateur en dur) | C | idem | T5 (iv) rouge (le premier appel rend `r`, le port tenu ailleurs) ; journal `m6.log`. Preuve que T5 joue dans ses propres dossiers, donc que deux copies ne s'y voient pas (V2-1) |
| **M5** — mutation : verrou pris **dans un sous-shell** — `port_libre` remis en `PORT=$( … )` autour de l'ouverture et du `flock` | C | idem | T5 (i) rouge (le verrou est relâché au retour : le `flock -n` de fond réussit) ; journal `m5.log`. C'est la preuve que V1-1 est fermé, non portée par T6 seul (probabiliste) |
| **T6** — deux copies simultanées (AC-C3) | C | une invocation sous `.gate.lock` et `mem-guard` | **rouge avant le code** : les deux copies sur `59fb28712`, même invocation — au moins une sans résumé ou en échec (collision dans la fenêtre de 30 s du crochet), journal `t6-rouge.log` ; si deux tirages ne reproduisent pas, le dire au Dev Agent Record (le rouge reste porté par M4 et M5) |

## Tâches telles qu'écrites

- [ ] **Lot C** — T6-rouge (deux copies sur le code d'origine, une invocation) ; D-C1 (sites `:394`, `:395` réécrits) ; T5 ; M4, M5, M6 ;
      T6 vert ; D-C3 (dont la valeur périmée de `docs/ci.md:101`), D-C4. Commit `refs #1499`.

## Note de développement

- **Lot C** : `exec {fd}>"$f"` alloue un descripteur ≥ 10 ; il n'existe que dans le shell qui l'a ouvert — d'où l'appel direct
  (D-C1, point 1). Garder les descripteurs dans un tableau global pour que rien ne les ferme avant la fin. Ne pas toucher
  `scripts/recette-montee.sh` (le script de la release) : seul son test change. Les quatre `D=$(grep …)` (`:589`, `:755`, `:858`,
  `:875`) **ne se touchent pas** ici (I-1).

## Constats ouverts au moment du retrait (passe ciblée P3, Sonnet, `validation-p3-ciblee.md`) — à intégrer à la future fiche

- **V3-1 (MEDIUM)** — sous `set -euo pipefail`, le nettoyage prescrit par AC-C2 s'arrête : `kill` sur un PID mort ou sur un tableau
  vide rend 1 ou 2, et `wait` après `kill` rend 143 ; la première commande en échec d'un `trap … EXIT` l'interrompt, si bien que
  `rm -rf "$WORK"` (`recette-montee.test.sh:20`) n'est plus exécuté et que la fin du bloc T5 arrête le test. **Remède** : chaque
  `kill` et chaque `wait` gardés (`kill "$pid" 2>/dev/null || true`, `wait "$pid" 2>/dev/null || true`), boucle
  `for p in "${T5_PIDS[@]:-}"` ; et **`rm -rf "$WORK"` en tête du trap**, avant tout `kill`.
- **V3-2 (LOW)** — `$!` d'un `flock -n f bash -c '…; sleep 60' &` est le PID de `flock`, parent de l'enfant qui tient le verrou :
  `kill $!` laisse un `sleep 60` orphelin qui garde le verrou. **Remède** : **tuer le groupe de processus** du tenant (le lancer
  sous `setsid` et tuer le groupe), ou faire tenir le verrou par le shell tenant lui-même (`exec 9>f; flock -n 9 && { touch prêt;
  exec sleep 60; }`), dont `$!` est alors le tenant. **Jamais `pkill`/`killall` à motif large** (`docs/agents-orchestration.md` § 2 ;
  la passe P3 en a fait l'expérience).
- **V3-3 (LOW)** — AC-C2 (ii) et (iv) définissent `q`/`r` comme « premier port libre à `ss` » alors que `port_libre` avance **de 2**
  (`:388-393`) : si 24330 (24350) écoute, `q` (`r`) vaudrait 24331 (24351), jamais candidat — faux rouge en (iv), (ii) vert sans
  verrou. **Remède** : `q`/`r` = premier **candidat de la chaîne à pas 2** libre à `ss`.
- **V3-4 (LOW)** — M6 ne rougit pas « seulement » (iv) : (i) et (ii) rougissent aussi quand `port_libre` ignore la variable. **Remède** :
  écrire « (i), (ii) et (iv) », ou isoler (iv).

## Ce qu'une future fiche doit encore trancher

- Si le **signal de recyclage** se reproduit (la mécanique des tenants de fond de T5 est la source des trois derniers défauts), une
  conception plus simple de T5 : tenir les verrous de test dans le shell principal lui-même (`exec {fd}>…; flock -n`), sans
  processus de fond, et sonder par `flock -n <fichier> true` (qui rouvre le fichier) — ce qui rend V3-1 et V3-2 sans objet.
- Le corps de #1499 a été ramené à la seule collision de ports (texte dans la fiche 49-5) ; I-1 porte l'arrêt sans résumé.
- **Estimation** (retirée de la fiche 49-5 avec le lot C) : 2 h 30, dont au moins 4 × 6 min de temps machine (T6 rouge, T6 vert, M4, M5, …). **Ordre** : le lot C coûte le plus en temps machine ; le lancer tôt dans une story à lui.

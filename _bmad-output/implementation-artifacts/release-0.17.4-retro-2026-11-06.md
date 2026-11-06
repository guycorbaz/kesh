# Rétrospective — la release 0.17.4

**Date : 2026-11-06.** Tenue **en autonomie** par l'orchestrateur, sans dialogue animé, sur demande de Guy
(« retrospective et revue de projet maintenant », 2026-11-06). Aucune décision du Project Lead n'est inventée ici : ce
qu'une rétrospective animée lui aurait demandé est au § 9, chaque question avec une recommandation. **À relire par
Guy** ; la revue de projet suit.

**Périmètre.** Du tag `v0.17.3` (2026-11-05 13:00:49 +01:00) au tag `v0.17.4` (2026-11-06 03:05:19 +01:00) : trois
stories hors epic (clés `v0174-session`, `v0174-mise-a-jour`, `v0174-correctifs` et leurs sous-fiches au
`sprint-status.yaml`), une PR de release (#924, squash `c2051805`). Mandat de Guy : « continuer de manière autonome
jusqu'à la création de l'image docker de la release. **Le but de cette release est plus de diminuer le nombre de bug
que d'implémenter de nouvelles fonctionnalités.** »

**Sources** : les fiches `v0174-*.md` (Change Logs, Dev Agent Records), les trois registres `v0174-choix-autonomes-*.md`,
les prompts versionnés ; hors dépôt, en lecture : `kesh-gate-logs/v0174{s,m,c}*`, `rel0174/`, `prompts/v0174-*.md` et
`rel0174-integration.md` ; `gh issue list` (instantané `kesh-gate-logs/retro0174/issues.json`, 2026-11-06, script
`retro0174/compte.py`) ; le journal de l'orchestrateur (`memory/epic-15-autonome.md`, `kesh-en-production.md`), cité
**« relevé de l'orchestrateur »** quand aucune source du dépôt ne porte le fait.

⛔ **Aucune donnée réelle de Guy n'est écrite ici** — le dépôt est public. Les contrôles faits sur les livres du NAS
sont décrits par leur nature, sans valeur.

---

## 1. Ce qui a été livré

| Story | Fiches | Contenu | Issues fermées par la PR |
|---|---|---|---|
| **v0174-session** | `-1` (serveur), `-2` (écran) | le jeton de renouvellement est stocké **haché** (empreinte SHA-256 au format UUID dans la colonne existante, sans migration ; anciens jetons effacés, une reconnexion) ; **avertissement avant la fermeture pour inactivité**, avec « Rester connecté » ; LOW de la 0.17.3 | #890 (P1), #895, #896, #898, #899 |
| **v0174-mise-a-jour** | `-1` (`-2` `superseded`) | **sauvegarde complète automatique avant toute migration** (`.keshbackup` format 2 dans `pre-upgrade/`), refus de démarrer si elle échoue, échappatoire `KESH_SKIP_PRE_UPGRADE_BACKUP=<version>`, jamais de suppression automatique — **la version minimale** choisie par Guy | #847 |
| **v0174-correctifs** | `-1`, `-2`, `-3` | import CAMT des références d'entité (`&amp;`…) avec reconnaissance des lignes déjà importées (#819) ; bornes du verrou de période dans le refus (#869) ; pays contrôlé (#817) ; verrou de compte et connexion rendue au pool propre (#842, #540) ; date de la 0.17.1 (#901) ; lots de paiement au-delà de 200 factures (#888) ; date du jour locale (#889) ; projet archivé dans l'édition d'un brouillon (#872) | #819, #869, #817, #842, #540, #901, #888, #889, #872 |

Fermée à part, le 2026-11-05, sur accord de Guy : #897 (doublon de #896). **Seize fermetures** en tout (§ 2).

**Aucune migration** : 89 au tag v0.17.3, 89 au tag v0.17.4 (`git ls-tree -r v0.17.4 crates/kesh-db/migrations/ |
grep -c '\.sql'`). Le retour à la 0.17.3 se fait sans restauration, **après `DELETE FROM refresh_tokens`** — sinon les
empreintes y redeviennent des jetons valables ; joué à la recette d'intégration, empreintes identiques
(`rel0174/boot-0173-retour-container.log`).

| | v0.17.3 | v0.17.4 |
|---|---|---|
| Stories `done` (dépôt) | 322 | **329** (sept clés : les sous-fiches et l'index `v0174-correctifs`) |
| Tests backend (nextest, gate de release) | 4739 (12 ignorés) | **4813** (12 ignorés) — `rel0174/gate-backend.log` |
| Tests backend, mode CI (`cargo test -j1`) | — | 4820 passés, 16 ignorés, 0 échec — relevé de l'orchestrateur, après `f5d40d7e` |
| Tests frontend (Vitest) | 2623 (211 fichiers) | **2696** (218) — `rel0174/front-test-unit.log` |
| E2E (réussis / échecs / ignorés) | 343 / 7 / 18 | **351 / 8 / 18** — les 7 de la KF-029 et `sidebar-navigation.spec.ts:81`, vert rejoué seul (`rel0174/e2e-rejeu-sidebar.log`) |

**Diff** : `git diff --shortstat v0.17.3 v0.17.4` = 260 fichiers, +15 390 / −567 ; code seul (`-- crates frontend
scripts`) : 202 fichiers, +7 308 / −515.

**Publication** : tag annoté `v0.17.4`, `release.yml` vert (smoke `/health` = 0.17.4), `gcorbaz/kesh:0.17.4` =
`:latest` = `sha256:11fe599a…`. **En fonction sur le NAS depuis le 2026-11-06**, après une sauvegarde complète faite
**avant** la mise à jour cette fois (déclaration de Guy) ; `/health` du NAS rend `0.17.4` (constaté).

---

## 2. Les engagements, et l'objectif de la release

```sh
python3 kesh-gate-logs/retro0174/compte.py      # sur l'instantané issues.json du 2026-11-06
```

| Moment | Issues ouvertes | Bugs | P1 |
|---|---|---|---|
| Tag v0.17.3 | **239** | **87** | 14 |
| Tag v0.17.4 (fermetures du merge comprises) | **246** | **91** | 13 |
| Aujourd'hui (2026-11-06, après #926) | 247 | 91 | 13 |

Sur la période : **23 issues ouvertes** (#901, #903 à #923, #925 ; 14 bugs), **16 fermées** (10 bugs) ;
239 + 23 − 16 = 246, 87 + 14 − 10 = 91. Le décompte strict à la seconde du tag rend 259 / 100 : treize fermetures du
merge sont horodatées de 0 à 18 secondes après lui (02:05:19–37Z pour un tag à 02:05:19Z) ; la règle des rétrospectives
précédentes (fermées au merge = fermées au tag) rend **246 / 91**.

### 2.1 L'objectif de Guy : ❌ **manqué**

Moins de bugs était **le** but. Le compte monte de quatre. La release a pourtant fermé dix bugs, dont le seul P1 neuf
(#890) et cinq P2. **D'où viennent les quatorze bugs ouverts** :

- **treize naissent des fiches, des revues et de la recette de la release** : limites ou défauts voisins relevés et
  laissés hors périmètre (#903, #904, #906, #907, #908, #909, #911, #912, #913, #915, #916), constats de la recette et
  des revues de code (#920, #922) — #909 (P2, aucun écran ne corrige l'adresse de la société) et #911 (P3, sauvegarde
  chargée en mémoire) sont des défauts anciens que les fiches ont vus en lisant le code ;
- #901 vient de la rétrospective 0.17 ; ouverte et fermée dans la période.

Aucun bug de la période ne vient de l'usage de Guy : les défauts de production ont été ceux qu'il avait déjà signalés.
**Le mécanisme est celui des quatre releases précédentes** : « on trace toujours » fait ouvrir une issue pour chaque
limite qu'une fiche décide de ne pas traiter, et une release de correctifs, qui touche des zones anciennes, en
découvre beaucoup. Cela n'est pas un défaut de la règle — Guy veut les suivre —, mais **l'objectif « moins de bugs »
n'est pas atteignable tant que chaque fiche écrit ses limites en issues de type `bug` plus vite qu'elle n'en ferme.**
Deux leviers, au § 9 (Q1, Q2).

⚠️ Les trois issues de LOW de la release (#919, #921, #923) ne portent ni `bug` ni `technical-debt` (`P4` seul) :
elles ne comptent pas dans les bugs — ce que recommandait la Q2 de la rétrospective 0.17, appliqué sans que Guy l'ait
tranché, et sans le label `technical-debt` qu'elle proposait.

### 2.2 Aucun P1 ne passe : ✅ **pour le P1 neuf**

#890 est fermée. Restent treize P1 : les dix de la TVA (report assumé, § *Priorités des défauts*) et trois autres, dont
#235 (pièces justificatives, art. 957a CO), reportée depuis l'Epic 32.

### 2.3 Une fonctionnalité nouvelle : ✅

La sauvegarde avant migration (#847) et l'avertissement de session (#895), conformément à la priorité : deux
fonctions qui protègent la production, aucune fonction de confort.

---

## 3. Le processus

### 3.1 Validations — 15 passes

| Story | Passes | Tendance (C/H/M/L) | Modèles |
|---|---|---|---|
| session | 4 | P1 R 0/0/2/9 + F 0/0/3/13 → P2 0/0/6/9 (cinq MEDIUM **nés de la remédiation P1**) → P3 0/0/1/7 → P4 0/0/0/3 | Sonnet, Opus, Sonnet, Haiku ciblée |
| mise-a-jour | 7 | P1 R 0/2/8/7 + F 0/0/6/6 → P2 0/1/10/9 → P3 0/1/5/4 → *version minimale* → P4 0/2/5/7 → P5 0/0/3/7 → P6 0/0/3/4 → P7 0/0/0/4 | Sonnet, Opus, Sonnet, Opus, Sonnet, Haiku ciblée, Sonnet ciblée |
| correctifs | 4 | P1 R 0/1/2/8 + F 0/0/4/7 → P2 0/0/2/15 → P3 0/0/1/7 → P4 0/0/0/3 | Sonnet, Opus, Sonnet ciblée, Haiku ciblée |

**La mise à jour (#847) est le cas de la release.** P1 à P3 rendent chacune un HIGH, **tous recyclés** (le même défaut
sous une autre forme, ou né du correctif) : le signal de non-convergence. L'orchestrateur a d'abord simplifié (aucune
suppression automatique, une seule échappatoire), puis **posé la question à Guy**, qui a choisi la version minimale.
C'est **la leçon 1 de la rétrospective 0.17 appliquée** — mais une passe plus tard qu'elle ne le demandait : la
simplification de l'orchestrateur a coûté la P3, et la P4 a encore rendu deux HIGH nés de la réécriture. La boucle
s'est close en P7 ; dès la P6, l'orchestrateur n'a plus remédié lui-même (retraits et qualifications seulement).

### 3.2 Revues de code — 10 passes, aucun HIGH

| Fiche | Passes | Tendance |
|---|---|---|
| session-1, session-2 | 1 chacune | P1 Sonnet, trois lentilles : 0/0/0/3 (×3) et 0/0/0/4, 0/0/0/4, 0/0/0/7 |
| mise-a-jour-1 | 2 | P1 Opus B 0/0/1/4, E 0/0/1/5, A 0/0/3/2 → P2 Sonnet ciblée 0/0/0/2 |
| correctifs-1 | 3 | P1 Sonnet B 0/0/1/4, E 0/0/1/5, A 0/0/1/6 → P2 Sonnet ciblée 0/0/1/3 (le manuel taisait le double import silencieux : #922) → P3 Haiku ciblée 0/0/0/3 |
| correctifs-2 | 1 | P1 Opus 0/0/0/12 |
| correctifs-3 | 2 | P1 Sonnet, un MEDIUM (captures qui ne montraient pas l'état) → P2 Sonnet ciblée 0/0/0/1 |

**Le MEDIUM le plus utile est un MEDIUM de manuel** : la P2 de correctifs-1 a vu que le manuel promettait la
reconnaissance des lignes déjà importées sans dire ce qui arrive si la relecture « à la 0.17.3 » échoue — une ligne
peut alors être importée deux fois sans signal (#922, P2). *Le manuel, encore, est l'endroit où l'écart se voit.*

### 3.3 Gates et recette

- Gates complets au dernier commit de code de chaque story, puis sur la branche d'intégration : verts (chiffres au § 1).
- **Recette de montée et de retour** (A3 et A4 de la rétrospective 0.17, appliqués par le prompt d'intégration) : base
  peuplée par l'image 0.17.3 (relevé CAMT à entité, factures validée, payée, annulée, session ouverte), binaire 0.17.4 :
  aucune migration appliquée, aucun fichier `pre-upgrade/` écrit, empreintes identiques, `refresh_tokens` sans jeton
  en clair, ancien cookie refusé, réimport CAMT reconnu, `.keshbackup` 0.17.3 importé ; puis retour à l'image 0.17.3
  après vidage de `refresh_tokens` : empreintes identiques.
- **Recette AC-8 de #847** (base 0.15.0 peuplée par l'image publiée) : sauvegarde écrite avant migration, 83 → 89,
  0.15.0 refusée, restauration dans une 0.15.0 neuve, pièces identiques par empreinte. **La procédure de retour du
  manuel n'a pas été jouée telle qu'écrite** (interdit de toucher au MariaDB partagé) : #920.
- **Non établi** : l'aperçu PDF (la souris dans le document ne doit pas compter comme une activité) — non reproductible
  en navigateur sans tête ; laissé au contrôle de Guy.

---

## 4. Incidents

### 4.1 La CI rouge sur un défaut que le gate local ne pouvait pas voir

La CI de #924 a échoué au job backend : 17 tests `pre_upgrade_backup` en « DATABASE_URL must be set ». Cause :
`config::tests::reset_env` efface `DATABASE_URL` ; **nextest isole chaque test dans un processus, `cargo test` (la CI)
non** — l'état global effacé par un test atteint les suivants. Le gate local, vert, ne pouvait pas le voir.
Correctif de l'orchestrateur `f5d40d7e` (un verrou et une garde qui restaure la variable), reproduit d'abord en local en
mode CI (17 échecs sans, 648 verts avec, `cargo test -p kesh-api --lib -j1 -- --test-threads=1`), suite complète en
mode CI verte, revue ciblée Sonnet 0/0/0/2 ; CI verte au second cycle. Tracé : #925 (le gate local ne lance pas les
tests comme la CI).

*La règle « Test Locally First » promet « exactement les checks de la CI » ; pour les tests, c'est faux depuis que
nextest est le gate local.* Elle ne l'était pas par intention : le `CLAUDE.md` présente nextest comme une
« alternative » au `cargo test` sériel. Q3.

### 4.2 Un compte rendu faux de l'orchestrateur

Après la fusion, l'orchestrateur a annoncé à Guy six issues « non fermées » (#869, #872, #817, #842, #901, #540) : il
les avait lues **avant** que GitHub ait appliqué les `closes` du merge, ce qui se fait en différé. Ses `gh issue close`
ont tous répondu « already closed », sans commentaire ajouté ; la rectification a été faite auprès de Guy. *Une
vérification d'après fusion se fait après un délai, ou se refait avant d'être rapportée.*

### 4.3 Écarts des agents

- Fichiers parasites laissés par des agents dans `kesh-gate-logs/` (`.p3-man-7bd.pdf`, `.v0174c-p3-flat.txt`), retirés
  par l'orchestrateur.
- **Les six fiches livrées sont restées en `Status: review`** (et l'index `v0174-correctifs` en `ready-for-dev`) alors
  que le registre les dit `done` : le geste de clôture que la série 0.17 avait tenu (A2) n'était pas dans les prompts de
  cette release. Aligné par la PR de cette rétrospective.

### 4.4 La station

Aucun OOM (`journalctl -u systemd-oomd --since "2026-11-05 13:00"` sans ligne de victime). Le verrou commun
(`/tmp/station-heavy.lock`) a tenu pendant que la session mybibli travaillait, puis Guy a annoncé sa fermeture. Une
fausse alerte du moniteur (« Out of memory » d'un message du pilote NVIDIA) a fait resserrer son filtre.

---

## 5. Suivi des actions de la rétrospective 0.17 (§ 8 et § 10)

| # | Action | État au 2026-11-06 | Preuve |
|---|---|---|---|
| A1 | Verser les consignes à `docs/agents-orchestration.md` | ❌ | `git log v0.17.3..v0.17.4 -- docs/agents-orchestration.md` vide ; elles vivent dans `prompts/v0174-commun.md` |
| A2 | #890 en tête | ✅ | story v0174-session ; #890 fermée |
| A3 | Test de montée et de retour **dans un script du dépôt** | ⏳ | joué (§ 3.3) mais par des scripts hors dépôt ; `git grep "previously applied" v0.17.4 -- scripts` vide. L'échéance (« prochaine release qui porte une migration ») n'est pas encore venue |
| A4 | Les phrases des notes de version se jouent | ✅ en grande partie | le prompt d'intégration l'exigeait ; la recette les a jouées, sauf la procédure de retour de #847 (#920) et l'aperçu PDF, déclarés |
| A5 | Recette Claude Code C1-C3 ; issue qui la trace | ❌ | non jouée ; aucune issue ; neuf « non éprouvé » au manuel d'administration (`git grep -c`) |
| A6 | Registres de l'Epic 32 et *Status* `32-*` | ❌ | `epic-32-choix-autonomes.md` : 13 choix ; quatre fiches `ready-for-dev`, cinq `review` |
| A7 | Tracer les constats | ✅ | #897 fermée en doublon ; date de la 0.17.1 : #901, fermée par la release |
| A8 | Nettoyer `/tmp/kesh-e2e-*` | ❌ | **44** dossiers (`ls -d /tmp/kesh-e2e-* \| wc -l`) |
| A9 | Mettre à jour `serveur-mcp-decisions.md` | ❌ | la note annonce encore les propositions pour la 0.17.0 |
| Q3 | #890 en tête ? | ✅ répondue | périmètre accepté par Guy le 2026-11-05 |
| Q6 | Ce qui vient ensuite | ⏳ en partie | « une release courte d'abord » a été faite ; #235 et les propositions restent sans échéance |
| Q8 | Conteneur `kesh-rel0173-boot-0172` | ✅ | « oui pour les trois », supprimé |
| Q11 | Sauvegarde avant la 0.17.3 | ✅ répondue | « j'ai oublié de la faire » ; faite après. **Faite avant la 0.17.4** |
| Q1, Q2, Q4, Q5, Q7, Q9, Q10, Q12 | | — | sans réponse consignée |

**Bilan** : trois actions faites (A2, A4, A7), une en cours (A3), cinq non faites (A1, A5, A6, A8, A9). **Le constat
est le même pour la cinquième fois : ce qu'un prompt portait a été fait, le reste non.** Les cinq actions non faites
n'avaient pas de porteur dans la release.

---

## 6. Ce qui a marché, ce qui a coûté, les leçons

### 6.1 Ce qui a marché

- **Une release de correctifs livrée d'une traite en autonomie**, de l'accord du périmètre à l'image publiée, pendant
  que Guy tenait ses livres ; trois stories menées en parallèle dans trois worktrees, intégrées par un agent dédié.
- **La production protégée** : #890 (P1) fermée ; #847 installe le filet qui manquait depuis le 2026-10-31 ; la mise
  à jour du NAS s'est faite **après** une sauvegarde complète, la première fois consignée dans cet ordre.
- **La question à Guy plutôt que la passe de plus** (#847) : la version minimale a convergé en quatre passes après trois
  passes perdues.
- **Les revues de code** : dix passes, aucun HIGH, une seule fiche au-delà de deux passes.
- **La recette d'intégration**, montée et retour, empreintes identiques : chaque phrase de mise à jour des notes a été
  jouée, ou déclarée non jouée.
- **Le contrôle des livres du NAS** (2026-11-05), en lecture par l'API, après les suppressions de factures annulées
  faites par Guy : rien d'anormal relevé (relevé de l'orchestrateur, sans donnée ici).

### 6.2 Ce qui a coûté

- **L'objectif manqué** : +7 issues, +4 bugs, pour une release dont c'était le but.
- **Sept passes de validation** pour la version minimale d'une fonction de sauvegarde.
- **Un cycle de CI** perdu sur un défaut de test invisible au gate local.
- **Des gestes de clôture sans porteur** (§ 5), encore.

### 6.3 Leçons

1. **Pour faire baisser les bugs, il faut des fiches qui ne laissent pas de limites.** Une release « de correctifs »
   dont les fiches écrivent leurs limites en issues `bug` n'a aucune chance de faire baisser le compte. La recherche
   d'exhaustivité des revues (énumérer les cas) est bonne pour la qualité, mauvaise pour le chiffre, et ce sont deux
   objectifs distincts. Le choix revient à Guy (Q1).
2. **Une simplification de l'orchestrateur ne remplace pas la question au Project Lead** : sur #847, elle a coûté une
   passe. Quand le signal de non-convergence se lève, la question part **avant** la simplification.
3. **Le gate local doit exécuter les tests comme la CI au moins une fois avant le push de la release** (`cargo test
   -j1 --test-threads=1`) : un état global partagé ne rougit que dans un processus unique.
4. **Une vérification après fusion attend la fin des traitements différés** de GitHub, ou se refait avant d'être
   rapportée.

---

## 7. Actions

| # | Action | Propriétaire | Échéance | Critère |
|---|---|---|---|---|
| B1 | **Gate de release en mode CI** : ajouter au prompt d'intégration (et à `scripts/test-fast.sh`, option `--ci-exact` ou équivalent) l'exécution `cargo test --workspace -j1 -- --test-threads=1` sur la branche d'intégration — #925 | Orchestrateur | Prochaine release | journal `rel…/ci-cargo-test.log` cité dans la PR |
| B2 | **Clôture des fiches dans le prompt d'intégration** : `Status: done` des fiches livrées, au même commit que le registre | Orchestrateur | Prochaine release | `grep -m1 '^Status:'` des fiches = registre |
| B3 | **Script de montée et de retour au dépôt** (reprise de A3) | Orchestrateur | Prochaine release qui porte une migration | un script de `scripts/` lancé par le prompt |
| B4 | **#920 : jouer la procédure de retour de #847 telle qu'écrite** sur un hôte jetable avec son propre MariaDB | Guy (hôte), orchestrateur (recette) | Avant la prochaine release à migration | #920 fermée |
| B5 | **Reprendre A1, A5, A6, A8, A9** en leur donnant un porteur, ou y renoncer par écrit (Q4) | Orchestrateur, sur décision de Guy | Prochaine story | décision écrite ou action faite |
| B6 | **CR #926** (numérotation des factures fournisseur) et **#918** (graphiques de la page d'accueil) : fiches à écrire après les réponses de Guy | Orchestrateur | Après les réponses | fiches validées |

---

## 8. La suite

Aucun epic n'est ouvert. Candidats, **sans ordre décidé** (§ 9, Q5) :

- **une release de défauts** : #922 et #909 (P2), les LOW mécaniques de #919, #921, #923, et une sélection de P2 et P3
  anciens (#729, #742, #755, #776, #803, #804, #805) — le seul levier sur le compte ;
- **#918**, les graphiques de la page d'accueil (réponses de Guy reçues, « après la 0.17.4 ») ;
- **#926**, la numérotation des factures fournisseur (questions sans réponse) — probablement une migration, donc la
  première à exercer #847 en production ;
- **#235** (P1, pièces justificatives) et les propositions de l'agent, reportées depuis l'Epic 32 ; #235 précède la 0.18 ;
- **CR #877** (décomptes de charges aux locataires), de la taille d'un epic.

**Décompte de départ** : 246 issues ouvertes, 91 bugs, 13 P1 au tag v0.17.4 (247 aujourd'hui avec #926).

---

## 9. Questions pour Guy

1. **Le compte des bugs, ou l'exhaustivité des fiches ?** Les fiches écrivent leurs limites en issues `bug` ; c'est ce
   qui fait monter le compte même quand une release corrige dix bugs. *Recommandation* : pour la prochaine release de
   défauts, demander aux fiches de **corriger** les limites voisines quand elles sont courtes, plutôt que de les tracer,
   et ne choisir que des issues dont la correction ne déborde pas ; garder « on trace toujours » pour ce qui déborde.
2. **Le label des issues de LOW** : #919, #921, #923 n'en portent aucun de type. *Recommandation* : `technical-debt`,
   pour qu'elles se trouvent par label sans compter comme bugs (Q2 de la rétrospective 0.17, encore ouverte).
3. **Le gate local en mode CI** (B1, #925) : faut-il l'imposer avant chaque push, ou seulement avant la PR de release ?
   *Recommandation* : avant la PR de release et après toute modification d'un test qui touche l'environnement ; le
   sériel complet coûte trop pour chaque push.
4. **Les actions sans porteur** (A1, A5, A6, A8, A9) : les faire porter par la prochaine release, ou y renoncer ?
   *Recommandation* : faire A1 et A8 (courtes, et A8 est un geste de ta main ou une permission), renoncer à A6 par
   écrit, garder A5 et A9 pour le moment où le serveur MCP reviendra au plan.
5. **Qu'est-ce qui vient ensuite ?** *Recommandation* : une release de défauts courte (§ 8, premier point, avec la
   règle de Q1), puis #918 ; #926 dès tes réponses ; et décider l'échéance de #235.
6. **Les questions de la rétrospective 0.17 restées sans réponse** (Q1 objectif d'issues, Q2, Q4 verrou commun au
   `CLAUDE.md`, Q5 règle de la release corrective, Q7, Q9, Q10, Q12) : veux-tu les reprendre une à une, ou que je
   propose une décision par défaut pour chacune ?
7. **#869** : faut-il ajouter « et l'exercice » au message de période verrouillée, maintenant que les bornes y figurent ?

[#847]: https://github.com/guycorbaz/kesh/issues/847
[#890]: https://github.com/guycorbaz/kesh/issues/890
[#920]: https://github.com/guycorbaz/kesh/issues/920
[#922]: https://github.com/guycorbaz/kesh/issues/922
[#925]: https://github.com/guycorbaz/kesh/issues/925
[#926]: https://github.com/guycorbaz/kesh/issues/926

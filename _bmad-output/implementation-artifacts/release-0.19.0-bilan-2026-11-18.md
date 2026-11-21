# Bilan de release — Kesh 0.19.0

**Date : 2026-11-18, le soir du tag `v0.19.0`.** Bilan court, tenu **en autonomie** : rien n'a été décidé à la place de
Guy ; ce qui demande sa décision est au § 7. Ce n'est pas une rétrospective : les Epics 42, 43 et 44 restent ouverts et
continuent en 0.20.0 ; l'Epic 37 (multi-devises, responsable de l'epic) a déjà sa rétrospective, `epic-37-retro-2026-11-17.md`, qui n'est
pas refaite ici.

**Périmètre.** De `v0.18.0` (tag annoté 2026-11-16T17:45:46+01:00, squash `8b50fd195`) à `v0.19.0` (tag annoté
2026-11-18T18:32:16+01:00 sur le squash `773df80da`, PR #1276). Image `gcorbaz/kesh:0.19.0` ; smoke `/health` de
`release.yml` = 0.19.0 (run 44359782378) : relevé du journal de projet, **non recontrôlé ici**. Publiée le **mercredi 18**
au lieu du vendredi 20 prévu, la coupure ayant été avancée au 18 (décision de Guy).

**Sources** : `sprint-status.yaml` (au commit `773df80da`) ; les Change Logs des fiches `36-16a-*`, `43-*`, `44-1a-*`,
`v0190-*` ; les registres `e0190-choix-autonomes-*.md`, `e44-choix-autonomes-44-1.md` ;
`planning-artifacts/release-0.19.0.md` ; `kesh-gate-logs/rel0190/` (`choix-autonomes.md`, `pr-corps.md`,
`revue-integration-p*.md`, `build-1.log`) ; le journal du dossier de projet pour la chronologie ;
`gh issue list --state all --limit 1500` (lecture seule, le 2026-11-18 au soir).

⛔ **Aucune donnée réelle de Guy n'est écrite ici** — le dépôt est public.

---

## 1. Ce qui a été livré, ce qui est reporté

| Story | Issue | Objet |
|---|---|---|
| 37-1 à 37-5 (Epic 37, PR #1247, fusionnée le 17) | #233 (en partie) | comptabilité multi-devises — voir sa rétrospective |
| 42-1 (= 36-16a) | #1167 (en partie) | changer le compte analytique d'une facture client, même payée |
| 43-1a, 43-1b | #823 | règlement groupé : modèle, lettrage par ligne, clients puis fournisseurs ; porte la migration `20261118000001_grouped_settlements` |
| 43-2, 43-3 | #823 | API du règlement groupé, puis l'écran (une ligne du relevé, factures ajoutées une à une) |
| 43-7 | #823 | plafond de l'écart réglable par l'administrateur (5.00 par défaut) |
| 43-6 | #823, #1167 | refus `409 PROJECT_REASSIGNMENT_GROUPED_SETTLEMENT` : on ne réimpute pas une facture réglée en groupe |
| 44-1a | #944 (en partie) | QR-facture : grille SIX, *Payable by*, largeurs mesurées |
| v0190-correction | #1162 | le client affiché est celui qui est enregistré |
| v0190-nom-societe | #1097 | le nom de la société devient modifiable |

**Quatre migrations** (91 → 95 : `ls crates/kesh-db/migrations/*.sql | wc -l` à la tête, `git ls-tree v0.18.0` au tag
précédent) — trois de l'Epic 37, une de l'Epic 43 —, aucune ne relève `kesh_version_min_required`, qui reste à 0.17.6.
Stories `done` (règle du `CLAUDE.md`, parseur YAML) : **390 → 408** ; les 18 comptent la même story deux fois, sous
`36-16a` et sous `42-1`.

**Fermées au tag** : #823, #1162, #1097 (et #1237, un CR fermé hors release). **Restent ouvertes** : #1167, #944, #233.

**Reporté en 0.20.0** (`backlog`, coupure du 2026-11-18) : 42-2 et 42-3 (réimputation d'une facture fournisseur et d'une
écriture manuelle ; sorties dès le 17, elles dépendent de la colonne de l'Epic 43), 43-5 (paiements non rapprochés,
reprise dans l'épique « envoyée », #1236), 44-1b (paliers intermédiaires de la cascade, #1230) ; les trois stories de
solde 42-4, 43-4 et 44-2.

## 2. Ce qui a marché

- **#823 entière publiée**, cas clients et fournisseurs, avec sa migration, deux jours après son kickoff : la priorité
  que Guy lui avait donnée a tenu, et le découpage en stories « qui sortent une à une » a permis une coupure propre.
- **Deux développeurs, une release** : l'Epic 37 fusionnée en premier dans `main` (le 17, décision de Guy), l'Epic 43
  reprise par-dessus pendant son développement ; aucune migration en conflit d'ordre.
- **La recette de montée** depuis l'image 0.18.0 (`recette-montee-1.log`, `recette-complement-4.log`) : quatre
  migrations, sauvegarde `pre-upgrade/`, refus de la 0.18.0 sur la base migrée, retour par la sauvegarde, et règlement
  groupé client et fournisseur joué **par une clé d'API** sur une base peuplée en 0.18.0.
- **La revue du code d'intégration** a trouvé ce que les revues de story ne voient pas (7 MEDIUM, tous dans les textes :
  manuel, guide, CHANGELOG) et s'est close en trois passes, plus une ciblée pour la 43-6.
- **Gates à la tête finale** (`b9df3e146`, avant le squash) : gate-ci 6150 passés / 0 échec, Vitest 4612, E2E 480
  passés et les sept échecs de la KF-029 seuls (`pr-corps.md`, journaux suffixés `-436`).

## 3. Ce qui a coûté

### 3.1 Les passes de revue

Notation CRITICAL/HIGH/MEDIUM/LOW, lue aux Change Logs ; « 3 lentilles » = B, E, A.

| Story | Validation de la fiche | Revue de code |
|---|---|---|
| 36-16a / 42-1 | 3 : Sonnet 0/2/7/7 → Opus 0/0/4/6 → Sonnet ciblée 0/0/0/3 | 2 : 3 lentilles (Opus, Sonnet, Opus) 0/0/2/13 → Sonnet ciblée 0/0/0/5 |
| 43-1a | 5 : Opus sur la 43-1 0/3/7/8, puis **découpage** → 0/3/4/8 → 0/0/6/10 → Haiku ciblée 0/0/1/6 → Haiku ciblée 0/0/0/4 | 3 : 3 × Opus 0/1/6/19 → 0/0/1/7 → ciblée 0/0/0/3 (modèles des P2 et P3 non déclarés) |
| 43-1b, 43-7 | contre `t3.md`, puis une passe ciblée commune : 0 au-dessus de LOW | 43-1b, 3 : 3 × Opus 0/0/3/≈12 → Sonnet 0/0/0/8 → Haiku ciblée 0/0/0/4 ; 43-7, 5 : 3 × Opus 0/0/1/≈10 → **Haiku ×3** ciblées 0/0/1/2, 0/0/1/3, 0/0/0/1 → Sonnet ciblée 0/0/0/1 |
| 43-2 | 5 : Sonnet 0/2/7/8 → Opus 0/0/6/6 → Sonnet 0/0/3/6 → Opus 0/0/2/6 → Sonnet 0/0/0/5 | 4 : 3 × Opus 0/1/6/~30 → Sonnet 0/1/2/5 → Opus 0/0/1/7 → Sonnet ciblée 0/0/1/3 (MEDIUM transféré à la 43-3) |
| 43-3 | 5 : Sonnet 0/3/6/8 → Opus 0/0/1/7 → Sonnet 0/0/1/6 → Opus 0/0/1/6 → Sonnet 0/0/0/5 | 4 : 3 × Opus 0/0/7/~30 → Sonnet 0/0/1/7 → Haiku ciblée 0/0/0/5 → Sonnet ciblée 0/0/0/3 |
| 43-6 | 2 : Opus 0/0/2/6 → Sonnet 0/0/0/2 | 2 : 3 × Opus 0/0/0/8 → Sonnet ciblée 0/0/0/1 |
| 44-1a | 5 : Sonnet 0/1/5/6 → Opus 0/0/3/7 → Sonnet 0/0/2/7 → **Haiku ×2** ciblées 0/0/2/2, 0/0/0/4 | 2 : 3 × Sonnet 0/0/0/15 → Haiku ciblée 0/0/0/2 |
| v0190-correction | **8, au plafond** : 0/0/3/8 → 0/0/1/7 → 0/0/1/6 → **Haiku ×2** ciblées 0/0/2/1, 0/0/2/3 → 0/0/1/9 → 0/0/1/3 → 0/0/2/2 ; deux MEDIUM recontrôlés à l'étape 0 | 3 : 3 × Opus 0/0/0/15 → Sonnet 0/0/0/4 → Haiku ciblée 0/0/0/2 |
| v0190-nom-societe | 4 : Opus 0/0/4/5 → Sonnet 0/0/0/6 → Opus ciblée 0/0/1/1 → Haiku ciblée 0 | 2 : 3 × Sonnet 0/0/0/8 → Haiku ciblée 0/0/0/3 |
| Intégration | — | 4 : 3 lentilles 0/0/7/16 → Sonnet 0/0/0/4 → ciblée 0/0/0/2 → Sonnet ciblée (43-6) 0/0/0/1 |

**Tendance** : les HIGH ne sortent qu'aux premières passes de validation et à la P1 de revue des 43-1a et 43-2 ; toutes
les boucles convergent, une seule au plafond (v0190-correction, petite story frontend, huit passes de validation pour un
composant de liste). **Écart à la règle « modèle différent de la passe précédente »** : des passes ciblées Haiku de suite
dans quatre boucles (43-1a, 43-7, 44-1a, v0190-correction) ; la 43-7 le déclare et l'a rattrapé par une passe Sonnet.

### 3.2 Les incidents

- **La 43-6 sortie par erreur à la coupure**, puis réintégrée. La consigne de coupure de l'orchestrateur la faisait
  sortir, alors que C-43-18 (fichier d'épique) interdit de publier la 42-1 et la 43-1a sans elle : la réimputation de la
  42-1 aurait réécrit toutes les lignes d'une écriture de règlement groupé (P1). L'agent d'intégration l'a vu, n'a rien
  tranché et l'a remonté (C-rel0190-3) ; Guy a choisi de la développer le jour même. Coût : une story de plus le jour du
  tag, une fusion et une chaîne de gates rejouée.
- **Plantage de la station à 10 h 46** (systemd-oomd) : construction Rust de la 43-6, build frontend et E2E d'un autre
  projet en même temps ; terminal et session tués, redémarrage vers 10 h 52. Rien de perdu au dépôt (`git fsck` sain,
  rien de non commité) ; les bases MariaDB en tmpfs perdues et reconstruites. Plus tôt, une construction complète en
  quatre jobs avait déjà été tuée au plafond de mem-guard (`build-1.log`, rc 137). Reprise en `-j 2`, une commande
  lourde à la fois (C-rel0190-6, -11).
- **KF-063 (#1261)** rouge au premier gate-ci de clôture de la 43-6, vert au rejeu : intermittence connue, tracée,
  ouverte.
- **La coordination avec l'agent du contributeur** : courriels entre agents (autorisés le 18), un défaut d'échappement CSV signalé
  hors issue publique (avis de sécurité privé à traiter en 0.20.0, décision de Guy), une fusion de #1247 en contournant la
  protection de branche (CI du fork sans secrets, copie #1254 verte). Puis le compte `agentizedbeat` a ouvert une
  cinquantaine d'issues en une demi-heure et **GitHub l'a restreint** : ses 56 issues (#1266–#1271, #1277–#1326) sont
  masquées (404).
- **Dates décalées en route** : coupure du jeudi soir avancée au mercredi, un rectificatif « quatre migrations, non
  cinq » envoyé à l'agent du contributeur après coup (C-rel0190-1).

## 4. Décompte au tag

Reconstitué depuis `createdAt`/`closedAt` de `gh issue list --state all --limit 1500 --json
number,createdAt,closedAt,labels` (881 issues rendues, la plus haute #1275), **avec les étiquettes actuelles** — une issue
réétiquetée depuis compterait autrement à sa date : c'est une approximation.

| | v0.18.0 (2026-11-16T16:45:46Z) | v0.19.0 (2026-11-18T17:32:16Z) | Écart |
|---|---|---|---|
| Issues ouvertes | 458 | **496** | +38 |
| Bugs (`bug` ou `known-failure`) | 236 | **247** | +11 |
| P1 | 14 | **14** | 0 |

Les décomptes de l'orchestrateur (458/236/14 et 496/247/14) sont **retrouvés à l'identique**. Entre les deux tags :
**42 issues ouvertes** (#1226 à #1275, hors PR et hors masquées : 14 `enhancement`, 13 bugs, 12 `technical-debt`,
3 `documentation` ; par priorité 25 P4, 12 P3, 4 P2, une sans priorité), **4 fermées** (#823, #1162, #1097, #1237).

**Cause de la hausse** : les revues des stories et de l'intégration tracent leurs LOW hors périmètre, une issue par
problème, comme la règle le veut ; s'y ajoutent les CR de Guy du 17 (#1236 état « envoyée », #1242 types de lignes) et
les regroupements de menus proposés (#1249 à #1253). Moins d'issues au tag était un objectif, non une barrière : il est
manqué de 38.

**Hors décompte : les 56 issues masquées** de `agentizedbeat`. L'API ne les rend plus ; elles ne sont dans aucun chiffre
ci-dessus. Les six premières (#1266–#1271) avaient été fermées par l'orchestrateur en citant #1247 (journal de projet) ;
pour les cinquante autres (#1277–#1326), leur date de création par rapport au tag n'est pas vérifiable tant qu'elles sont
masquées : si certaines sont antérieures à 17:32:16Z et ouvertes, le décompte au tag est **sous-estimé d'autant**. Le
corps de PR comptait 505 issues l'après-midi : l'écart de 9 avec 496 tient aux trois fermetures de la fusion et, très
probablement, aux six #1266–#1271 alors visibles.

## 5. Les trois engagements (`CLAUDE.md` § Priorités des défauts)

1. **Le nombre de bugs baisse : non** (236 → 247). Deux bugs anciens fermés (#1162, #1097), treize ouverts, tous nés des
   revues ou des corrections de la release (#1226, #1229, #1231 à #1234, #1238, #1239, #1243, #1245, #1261, #1262, #1263),
   dont deux P2 (#1226, #1245).
2. **Aucun P1 ne passe : non**, les quatorze passent, aucun ouvert ni fermé. Les **dix de la TVA** (#390 à #397, #401,
   #525) : report assumé, Guy n'étant pas assujetti. **#1198** (solde du reste et compte de TVA due, même famille que
   #525) : proposé au report TVA à la rétrospective de l'Epic 36 (Q4). **#235** (pièces justificatives) : Epic 41, avant
   la clôture de l'exercice 2026. **#404** (rapports et borne d'exercice) et **#98** (signature électronique) : restent P1,
   décision de Guy du 2026-11-14. Aucun n'était dans le périmètre de la 0.19.0.
3. **Une fonctionnalité nouvelle : oui**, et la plus attendue — #823 (un mouvement bancaire règle plusieurs factures) —,
   plus le multi-devises, la réimputation analytique d'une facture client et le nom de la société modifiable.

## 6. Actions proposées

| # | Action | Propriétaire |
|---|---|---|
| A1 | À chaque coupure, relire les contraintes de couplage des fichiers d'épique (« ne sort jamais si… ») avant de sortir une story, et les citer dans la consigne de coupure | orchestrateur |
| A2 | Écrire aux consignes communes des agents : constructions en `-j 2`, une commande lourde à la fois, `mem-guard --status` avant chaque étape lourde (ce qui a tenu après le plantage) | orchestrateur |
| A3 | Recompter le décompte au tag quand les issues masquées redeviennent visibles, et l'annoter ici, daté | orchestrateur |
| A4 | Dans les prompts de passe ciblée, nommer le modèle de la passe précédente et interdire le même | orchestrateur |
| A5 | En-tête de la fiche 43-7 : « Validation : à lancer » (lignes 10-11) est périmé : la validation est terminée (fiche 43-1b, ligne 13) — à corriger avec le prochain commit qui touche l'Epic 43 | orchestrateur |
| A6 | Mettre le NAS en 0.19.0 après la **sauvegarde complète** (base, `.keshbackup`, dossier des documents) : quatre migrations, retour par cette sauvegarde seulement | Guy |

## 7. Questions pour Guy

- **Q1 — Les 56 issues masquées.** Si GitHub ne rétablit pas le compte de l'agent du contributeur, faut-il les faire recréer
  (par qui, sous quel compte) ou les laisser perdues ? Et une fois visibles, veux-tu que le décompte au tag soit corrigé
  ici par annotation datée ?
- **Q2 — #1198.** La Q4 de la rétrospective de l'Epic 36 (verser #1198 au report assumé de la TVA) est-elle tranchée ?
  Sinon, je recommande toujours le report, nommé avec les dix autres.
- **Q3 — Passes Haiku de suite.** La règle « modèle différent de la passe précédente » vaut-elle aussi entre deux passes
  ciblées ? Quatre boucles de cette release l'ont lue autrement ; je propose de l'écrire au `CLAUDE.md`.
- **Q4 — La 42-1 sous deux clés.** `36-16a` et `42-1` sont toutes deux `done` : le décompte des stories (408) compte la
  même story deux fois. Faut-il passer l'une des deux à un statut qui ne compte pas, ou annoter le décompte ?
- **Q5 — Le plafond de mem-guard.** Le `CLAUDE.md` dit 10G + 1G et deux exécutions à la fois ; le journal de la release
  montre 8G + 1G et trois places (`build-1.log`). Lequel fait foi ?
- **Q6 — La semaine.** La 0.19.0 sortie le mercredi compte-t-elle pour la semaine, et la 0.20.0 garde-t-elle le vendredi
  2026-11-27 ?

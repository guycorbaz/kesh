# Story 25.1c-b2 : Ce que l'écran du journal d'audit rend faux — manuels, README, vocabulaire

Status: ready-for-dev

⚠️ **RÉOUVERTE le 2026-09-15 au soir**, après sa validation en 7 passes : les arbitrages qui ont rouvert la
25-1c-a changent la section « Traçabilité » et le manuel d'administration (cf. « Réouverture » au Change
Log). **Revalidation ciblée en cours.**

⚠️ **Issue du SPLIT de la 25-1c-b** (arbitrage du Project Lead du 2026-09-15, après la passe 2 de
validation) :

| | objet | état |
|---|---|---|
| 25-1c-zero | la colonne `audit_log.company_id` | done — PR #437 ouverte |
| 25-1c-a | la route de consultation, son vocabulaire traduit et son export CSV — backend | ready-for-dev, réouverte |
| 25-1c-b1 | l'écran : feature, page, garde, menu, gardes i18n, tests | ready-for-dev, réouverte |
| **25-1c-b2** *(celle-ci)* | **les textes : manuels, README, vocabulaire « journal d'audit »** | ready-for-dev, réouverte |

⛔ **Livrée dans la MÊME PR que la 25-1c-b1, et implémentée APRÈS elle** sur la même branche : les
manuels décrivent l'écran **tel qu'il est construit** (menu, rôles, filtres), et ne se relisent contre
le code qu'une fois ce code écrit. C'est cette PR qui porte `closes #378`.

⛔ **Dépendance à la 25-1c-a** : elle réécrit elle-même trois passages du manuel d'administration
(`admin-manual.tex:1585`, `:1590`, `:1803-1810`) et la phrase de `:1786`. **Les numéros de ligne de cette
fiche sont ceux de la branche `story/25-1c-b-journal-audit-ecran` au 2026-09-15** — empilée sur la
25-1c-zero, **avant** la 25-1c-a et la b1, et donc déjà différente de `main` (la 25-1c-zero a réécrit
`admin-manual.tex:1786`). Chaque site se retrouve **par sa phrase citée**, et se relit après le rebase.

## Story

**En tant que** comptable, administrateur ou lecteur de la documentation,
**je veux** que les manuels, le README et les libellés disent ce que le logiciel fait — un journal
d'audit **consultable** à l'écran, nommé d'un seul mot partout,
**afin de** ne pas lire qu'une fonction manque alors qu'elle existe, ni trois noms pour une même chose.

**Couvre** : [#378], volet documentation.

## ✅ Arbitrages du Project Lead — 2026-09-15 (`epic-25-vague1-suite.md`)

- **Vocabulaire** : **« journal d'audit »** — `Audit-Protokoll`, `registro di audit`, `audit log`.
- **Qui consulte** : **Comptable** et **Admin** ; ni Consultation, ni clé API.
- **Découpage** (soir du 2026-09-15), cité : *« **25-1c-b2** (les textes : manuels, README, alignement du
  vocabulaire « journal d'audit » dans les catalogues et le manuel) — livrées dans la même PR »*.
  ⇒ **l'alignement du manuel est ARBITRÉ**, pas décidé par cette story (AC 3).
- **Au soir du 2026-09-15** (`epic-25-vague1-suite.md` § *fin de soirée*) : **filtre strict** par société
  — il n'existe pas d'entrée sans société à expliquer ; l'**export n'est pas inscrit au journal** ; les
  **types d'entité et les actions sont traduits**, à l'écran comme dans le CSV, dans la langue de
  l'interface ; les lignes `:1796` et `:1778` du manuel d'administration se **corrigent** dans cette story
  (réponse *« corrige »*, AC 6) ; `:1797`, trouvée ensuite, se corrige **par décision de la story**.

## Acceptance Criteria

### Volet A — le vocabulaire

**1. Le glossaire** — `docs/i18n-glossaire.md`, **partie A**, une ligne :
« journal d'audit » | `Audit-Protokoll` | `registro di audit` | `audit log` | précédents **déjà livrés** :
`export-global-content-excludes` (`fr-CH:1241`, « journal d'audit interne » ; `en-CH:1184`, « internal
audit log ») et `fiscal-year-reopen-confirmation-body` (`de-CH:729`, « Audit-Protokoll ») ; **aucun en
italien** — la forme `registro di audit` vient de l'arbitrage du 2026-09-15.

⚠️ **Ne pas citer `nav-audit-log` comme précédent** : la clé naît dans la même PR, et le glossaire pose
lui-même qu'une clé qu'on écrit n'est pas un précédent (ligne « modèle (d'e-mail) »).

**Ce qui impose quoi — les trois sources ne se confondent pas** :

| périmètre | source |
|---|---|
| les catalogues (AC 2) | **règle 3 du glossaire** (§ *Comment s'en servir*) : *« Un terme de la partie A ne se change pas dans une story de rollout. Il se change dans une story qui met AUSSI à jour toutes ses occurrences déjà livrées — sinon le produit dit deux mots pour une chose. »* **La ligne du glossaire ne s'écrit pas sans l'AC 2.** |
| les manuels (AC 3) | **l'arbitrage du soir** (« dans les catalogues et le manuel »). La règle 3, à la lettre, ne vise que les clés, et le glossaire distingue même le vocabulaire du manuel de celui de l'interface (`i18n-glossaire.md:180`) |
| la brochure et le README (AC 3) | **une extension décidée par cette story** — motif : le manuel et le README nomment l'écran « Journal d'audit », et ne peuvent pas désigner la même chose par un second mot |

**2. Les catalogues** — deux clés s'écartent du terme, **sur cinq cellules** — trois sur huit sont déjà conformes :

| clé | fr | de | it | en |
|---|---|---|---|---|
| `fiscal-year-reopen-confirmation-body` | « piste d'audit » ✗ | « Audit-Protokoll » ✓ | « pista di audit » ✗ | « audit trail » ✗ |
| `export-global-content-excludes` | « journal d'audit interne » ✓ | « internes Audit-Log » ✗ | « registro audit interno » ✗ | « internal audit log » ✓ |

⇒ les cellules ✗ **alignées**, accords et articles ajustés à chaque langue. Lignes : `fr-CH:766` et
`:1241`, `de/it/en-CH:729` et `:1184` — **à retrouver par la clé**.

⚠️ **Greper le mot `audit` seul, sans casse, jamais la locution** :
`grep -rniE "audit" crates/kesh-i18n/locales/*/messages.ftl | grep -vE ':[0-9]+:(nav-)?audit-log'` — ⚠️ **en excluant les clés
`^(nav-)?audit-log`**, dont le NOM contient le mot et qui, après la 25-1c-a, ajouteraient plus de 500 lignes
(vérifiées à part, ci-dessous). ⚠️ **Le filtre n'ôte que la LIGNE de clé** : les lignes de continuation
d'une valeur `audit-log-*`, ses attributs ou variantes, et les **en-têtes de blocs** neufs
(`# --- Journal d'audit — …`) sortent encore, et se trient comme « vérifiés à part ». Le catalogue français écrit l'apostrophe
**typographique** (`’`) : un motif `piste d'audit` rend zéro ligne sur `fr-CH:766`, vérifié à la
spécification ; l'allemand écrit `Audit` avec majuscule. **Triés, à laisser** :

- `bank-accounts-errors-has-transactions` (`fr-CH:986`, `de/it/en-CH:936`) — « audit comptable »,
  `Buchhaltungsprüfung`, « audit contabile », « accounting audit » : la **révision** des comptes, sans lien
  avec `audit_log` (`errors.rs:1933`). Autre sens, dans les quatre langues ;
- les commentaires `# Story 14-2 — … audit …` (`fr-CH:763`, `de/it/en-CH:726`) : commentaires, pas
  valeurs.

⛔ **Les clés neuves ne sont PAS exemptées : elles sont VÉRIFIÉES.** Après la b1, chaque clé
`^(nav-)?audit-log` dont la valeur **nomme le journal** (`nav-audit-log`, et les clés d'écran de la b1 —
titre, sous-titre, état vide, bouton d'export — ; parmi les 133 clés de la 25-1c-a **à la spécification**,
tout libellé qui nommerait le journal) porte **exactement** la forme de sa langue —
« journal d'audit » / `Audit-Protokoll` / `registro di audit` / `audit log`. Motif : la 25-1c-a renvoie au
« vocabulaire de l'arbitrage » sans en écrire les valeurs, et le modèle allemand le plus proche à recopier,
`de-CH:1184`, dit « Audit-Log ».

**Le procédé, exécutable** — en Python, **sur la VALEUR seule** : pour chaque `.ftl`, les lignes
`^((nav-)?audit-log[\w-]*)\s*=\s*(.*)`, et dans la valeur `audit|protokoll|registro|journal|piste|trail`,
sans casse ; puis tri à la main. ⚠️ **Une valeur Fluent peut continuer sur les lignes suivantes**,
indentées (patron `fr-CH:1330-1331`, `email-password-reset-body`) : la ligne de la clé rend alors une valeur
vide. Les lignes de continuation sont rattachées à leur clé avant la recherche. ⚠️ **Un grep sur « audit » ne suffit pas** : une variante qui ne contient
pas ce mot (« Protokoll » seul, « journal » seul) lui échapperait — un inventaire lexical se clôt sur ses
propres mots (passes 3 à 5).

**3. Les manuels, la brochure et le README** (sources : tableau de l'AC 1). L'inventaire a été établi
par **deux procédés**, et c'est leur réunion qui fait foi :

- **lexical** — `grep -rniE "piste d.audit|audit-trail|audit trail|audit log|piste de contr" docs/manual/fr/*.tex README.md`
  rend **exactement** les douze premières lignes du tableau, rien à écarter ;
- **par le concept** (passe 4, PDF aplatis) — les phrases qui **désignent le journal** sans employer ces
  mots : les cinq dernières lignes.

| fichier | ligne | texte actuel | procédé |
|---|---|---|---|
| `user-manual.tex` | `:301` | « pour préserver l'audit-trail » | lexical |
| `user-manual.tex` | `:995` | « journalisée dans l'audit-trail » | lexical |
| `user-manual.tex` | `:1589` | `\section{Traçabilité (audit-trail)}` | lexical |
| `user-manual.tex` | `:1591` | « une \textbf{piste d'audit} (\texttt{audit\_log}) » | lexical |
| `user-manual.tex` | `:1603` | « Cette piste d'audit est \textbf{insert-only} » | lexical |
| `user-manual.tex` | `:1739` | `\item[Audit-trail (piste d'audit)]` — entrée du glossaire | lexical |
| `admin-manual.tex` | `:1141` | « l'audit-trail (\texttt{invoice.emailed}, …) » | lexical |
| `admin-manual.tex` | `:1780` | `\subsection{Audit-trail (audit\_log)}` | lexical |
| `admin-manual.tex` | `:2184` | `\item[Audit-trail]` — entrée du glossaire | lexical |
| `marketing-brochure.tex` | `:319` | « Audit-trail centralisé pour la responsabilité fiduciaire. » | lexical |
| `README.md` | `:29` | « audit log » (liste des fonctionnalités, en français) | lexical |
| `README.md` | `:219` | « la **piste de contrôle survit désormais à l'import d'une sauvegarde** » | lexical |
| `user-manual.tex` | `:858` | « nouvelle trace d'audit » → « nouvelle entrée au journal d'audit » | concept |
| `user-manual.tex` | `:1684` | « L'\texttt{audit\_log} garde toujours la trace » — le nom de table y sert de nom au concept | concept |
| `admin-manual.tex` | `:1088` | « La demande reste tracée dans l'audit » | concept |
| `admin-manual.tex` | `:1143` | « laisse sa propre trace d'audit » → « sa propre entrée au journal d'audit » | concept |
| `admin-manual.tex` | `:1935` | « La piste existe et est conservée, mais elle ne voyage pas avec l'export » — « piste » seul, abrégé de « piste d'audit » | concept |

⇒ **« journal d'audit »** partout, soit **dix-sept sites**. **Triés autre sens, à laisser** (passes 4 et 5) :
`user-manual.tex:597` (« piste de correction »), `:1373`, `README.md:45`, `:213`, `:218` (« audit par
trois experts »), `marketing-brochure.tex:106`, `:209` (« Une piste pour les fiduciaires » — une piste commerciale), `:296`.
⚠️ En passe 4, le mot « piste » seul n'avait été cherché que dans les manuels :
`marketing-brochure.tex:209` a été trouvé en passe 5, par le rejeu du motif élargi de l'AC 8.

Précisions :

- **Le nom technique `audit\_log` ne change pas** là où il désigne la table (`:1591`, `:1780`, les
  champs) ; il change là où il sert de nom au concept (`:1684`).
- **Les deux entrées de glossaire deviennent « Journal d'audit (angl. audit log) »** — l'anglais
  **arbitré**, pas « audit-trail », que l'AC 2 retire du catalogue anglais — et **se déplacent à la
  lettre J** (les glossaires des manuels sont alphabétiques), **à côté d'une entrée voisine qui ne désigne
  pas la même chose** : dans `user-manual.tex`, **après** `\item[Journal]` (`:1754`, le journal des
  écritures) ; dans `admin-manual.tex`, **avant** `\item[Journal entry]` (`:2189`). La définition doit
  rendre la différence lisible.
- **Les renvois se suivent** : `user-manual.tex:1739` renvoie à la section « Traçabilité », dont le
  titre change (`:1589`) ; toute mention textuelle de ces titres se met à jour.

⛔ **L'inventaire se REJOUE après le rebase, sur le PDF aplati, avec un motif ÉLARGI** (AC 8) : un site
ajouté par la 25-1c-a ou la b1 n'y figure pas.

### Volet B — ce que l'écran rend faux

**4. Le manuel utilisateur, l'encadré « Ce que le logiciel ne fait pas encore à votre place »**
(`user-manual.tex:497-511`, du `\begin{keshwarning}` au `\end`) porte **cinq** énoncés : **quatre sont faux ou périmés, le premier est vrai mais incomplet** — il se
**nuance**, il ne se supprime pas :

| lignes | énoncé | état |
|---|---|---|
| `:498-499` | « la modification et la suppression d'une écriture sont refusées » | **vrai pour la route des écritures, à nuancer** : une écriture disparaît encore avec la facture validée qu'on supprime (ci-dessous) |
| `:499-502` | « Ce qui manque encore, c'est la consultation du journal d'audit depuis l'application » | **faux** : l'écran existe (25-1c-b1) |
| `:502-503` | « Le verrou reste par ailleurs annuel : aucun verrou de période plus fin […] n'existe encore » | **déjà faux** : la Story 24-4c a livré le verrou de période (PR #425) |
| `:505-506` | « Une seule exception, déjà en vigueur : une écriture déjà contre-passée ne peut plus être supprimée » | **périmé** : la route des écritures refuse désormais **toute** suppression. La seule disparition d'une écriture passe par la suppression d'une facture validée, et ce n'est pas l'exception que l'énoncé décrit |
| `:508-510` | « préférez la contre-passation […], tant que le logiciel vous laisse le choix » | **périmé** : la contre-passation est **imposée**, le logiciel ne laisse plus le choix |

⚠️ **Pas de formule absolue dans le texte de remplacement** : la suppression d'une **facture** validée
emporte son écriture — `crates/kesh-db/src/repositories/invoices.rs:1350` — ⚠️ le **repository**, et non
`crates/kesh-api/src/routes/invoices.rs`, homonyme — passe `enforce_immutability = false`, « seul site du dépôt » (commentaire
`crates/kesh-db/src/repositories/invoices.rs:1339-1340`, repris par `crates/kesh-db/src/repositories/journal_entries.rs:973-975`), résidu assumé que le commentaire du code rattache à #380 — **fermée le 2026-09-09** — et à #381,
ouverte. Le manuel utilisateur le décrit
déjà dans sa section sur la suppression d'une facture ; le nouveau texte ne doit pas écrire qu'une
écriture ne peut disparaître « en aucun cas ».

⇒ L'encadré se **réécrit**, ou se **remplace par une note**. Dans les deux cas, le texte **conserve le
premier énoncé, nuancé** : la route des écritures refuse modification et suppression, mais la suppression
d'une facture validée emporte son écriture. Il dit aussi la contre-passation imposée. Ce n'est plus un
avertissement « ne fait pas encore » : l'encadré n'annonce plus de fonction manquante.

**5. Le manuel utilisateur, section « Traçabilité »** (`user-manual.tex:1611-1616`) :
*« n'est pas encore consultable via un écran dédié dans l'interface (page de consultation prévue pour une
version ultérieure) »* → décrire l'écran, **tel que la 25-1c-b1 le construit**. Critère : **chaque cas où
l'écran rend une liste vide ou un refus sans explication se dit dans le manuel**.

- menu **Administration → Journal d'audit**, réservé au **Comptable** et à l'**Admin** ;
- le détail déplié, l'export CSV ;
- ⛔ **les dates** : le tableau affiche l'heure **locale** ; le filtre de période porte sur des **jours
  UTC** — une opération faite peu après minuit, heure suisse, appartient au jour UTC **précédent** ; le
  CSV écrit les heures en **UTC** ;
- **la langue** : les actions et les types d'entité s'affichent **traduits**, dans la langue de
  l'interface de l'installation — et le CSV les écrit dans la même langue (25-1c-a AC 12) ;
- **les filtres « type d'entité » et « action » sont des listes** de libellés (25-1c-b1 AC 5) : on
  choisit une action, on ne la tape pas ;
- **l'identifiant d'entité exige un type** : le champ reste inactif tant qu'aucun type n'est choisi ;
- **l'export est plafonné à 10 000 lignes** (25-1c-a AC 11) : au-delà, un message demande d'affiner
  les filtres ;
- **une action historique**, écrite par une version antérieure et absente de la liste, s'affiche sous la
  forme de son **code** (25-1c-a AC 16), et **n'est pas proposée dans la liste « Action »** (25-1c-b1
  AC 3 : seul un code déjà présent dans l'adresse de la page y est ajouté) — on la retrouve par le type et
  l'identifiant d'entité.

La phrase voisine sur le **centre de notifications** (`:1615-1616`) reste vraie et se conserve.

**6. Le manuel d'administration, `admin-manual.tex:1786`** — l'item « Champs » dit aujourd'hui, sur la
branche, que le journal *« ne se \textbf{consulte} pas encore~: aucune route ni aucun écran ne permet de le
lire depuis l'application --- c'est le sujet de l'issue \#378 »* (graphie LaTeX exacte : un grep de « ne se
consulte » sur le `.tex` ne trouve rien), et la 25-1c-a le réécrit en « une route existe, l'écran reste à
venir ». → **l'écran existe** : accès Comptable et Admin, **refusé aux clés API** (la route rejette un
jeton d'accès personnel), et **ni la consultation ni l'export ne s'inscrivent au journal** (25-1c-a AC 9
et 14) — un réviseur qui chercherait « qui a exporté le journal » doit savoir que le journal ne le dit pas.

✅ **Trois défauts voisins du manuel d'administration, CORRIGÉS DANS CETTE STORY — à deux titres** :

- `:1796` et `:1778`, trouvées en passe 4 : **par l'arbitrage du 2026-09-15 au soir** — la question portait
  sur ces deux lignes, la réponse fut *« corrige »* ;
- `:1797`, trouvée en revalidation R1 : **par décision de la story**, que l'arbitrage ne nomme pas. Motif : la
  nuance apportée à `:1796` rend `:1797` trompeuse, et l'une ne se corrige pas sans l'autre.

Les consignes :

- `:1796` — la formule absolue se **nuance** comme l'AC 4 le prescrit au manuel utilisateur : la route
  des écritures refuse modification et suppression quel que soit le rôle ; la suppression d'une facture
  validée emporte son écriture ;
- `:1778` — « Les 5 rôles standards » devient **trois**, en accord avec `:1345` ;
- `:1797`, **voisine de `:1796`** — « Il conserve aussi les modifications et suppressions **antérieures au
  gel** » laisse entendre, une fois `:1796` nuancée, qu'aucune suppression postérieure n'est journalisée ;
  or la suppression d'une facture validée inscrit `journal_entry.deleted` (`crates/kesh-db/src/repositories/invoices.rs:1350`,
  `crates/kesh-db/src/repositories/journal_entries.rs:1061`) → à reformuler dans le même geste.

| ligne | texte | ce que dit le code |
|---|---|---|
| `admin-manual.tex:1796` | « l'application refuse les deux, **sans exception et quel que soit le rôle** » (conformité OLICo) | faux : la suppression d'une facture validée emporte son écriture (`crates/kesh-db/src/repositories/invoices.rs:1350`) — la formule absolue que l'AC 4 interdit |
| `admin-manual.tex:1778` | « Les **5** rôles standards » | faux : trois rôles (`entities/user.rs:20-27`), ce que dit déjà `admin-manual.tex:1345` — et c'est la ligne qui précède la sous-section que l'AC 3 renomme |
| `admin-manual.tex:1797` | « Il conserve aussi les modifications et suppressions **antérieures au gel** » | trompeur une fois `:1796` nuancée : la suppression d'une facture validée journalise encore `journal_entry.deleted` après le gel (`crates/kesh-db/src/repositories/invoices.rs:1350`, `crates/kesh-db/src/repositories/journal_entries.rs:1061`) |

**7. Le README**, feuille de route :

- `README.md:218` — « le journal d'audit n'est consultable par aucun écran » ;
- `README.md:219` — « la **consultation depuis l'application** ([#378]), qui n'existe toujours pas ».

→ mis à jour **dans le même commit** que l'écran (§ *Synchroniser le planning du README*), ainsi que les
deux sites de vocabulaire de l'AC 3.

### Volet C — le contrôle

**8. Régénération et vérification** :

- `make fr` dans `docs/manual/`, PDF commités ;
- **PDF aplati vérifié** (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`) — les anciennes phrases
  absentes, les nouvelles présentes ;
- **inventaire rejoué sur les PDF aplatis et le README**, motif élargi puis tri à la main — **en
  Python** : `grep` est ici `ugrep`, qui refuse ce motif avec son contexte `{0,60}`. Motif, sans casse :
  `piste|trace d.audit|dans l.audit\b|audit-?trail|audit log`.
  - **Recoller d'abord les césures** du texte aplati (`re.sub(r"(\w)- (\w)", r"\1\2", t)`) : le PDF du
    manuel d'administration coupe des mots en fin de ligne, et un « jour- nal » ferait passer à tort le
    contrôle « nouvelles phrases présentes ».
  - Une occurrence peut apparaître **plusieurs fois** dans un PDF (sommaire, en-tête de page) : on compare
    des **sites**, pas des nombres.
  - **Reste attendu, et seulement lui, écrit en entier** :
    - les deux « (angl. audit log) » des glossaires ;
    - `user-manual.tex:597` (« piste de correction ») ;
    - `marketing-brochure.tex:209` (« Une piste pour les fiduciaires »).

  Les six autres sites triés « autre sens » ne contiennent que « audit » ou « auditable » : le motif ne
  les rend pas ;
- ⛔ en LaTeX le souligné s'écrit `\_` : un grep de `audit_log` sur le `.tex` ne voit rien.

`website/index.html:106` et `roadmap.html:105` (« an audit log that a backup import no longer
replaces ») **restent vrais**, emploient déjà le terme `en-CH`, et ne se touchent pas.
`marketing-brochure.tex:139` dit déjà « journal d'audit ».

**9. Les catalogues passent leurs gardes** : `cargo test -p kesh-i18n` (parité des quatre locales,
`loader.rs:693`) ; `npm run test:unit` — les deux clés de l'AC 2 changent de **valeur**, pas de nom, et
aucun compteur ne doit bouger ; s'il bouge, l'écrire.

## Tasks / Subtasks

- [ ] **T0 — Après la 25-1c-b1**, sur la même branche : relire chaque site **par sa phrase** ; rejouer
      l'inventaire de l'AC 3 (motif élargi de l'AC 8) et le grep de l'AC 2.
- [ ] **T1 — Glossaire et catalogues** (AC 1, 2), dont la vérification des clés neuves.
- [ ] **T2 — Vocabulaire des manuels, de la brochure et du README** (AC 3), dix-sept sites.
- [ ] **T3 — Encadré et section « Traçabilité » du manuel utilisateur** (AC 4, 5) — contrôler chaque
      affirmation neuve **contre le code de la b1**, pas contre cette fiche.
- [ ] **T4 — Manuel d'administration et README** (AC 6, 7), dont les trois défauts voisins
      (`:1796`, `:1797`, `:1778`).
- [ ] **T5 — Régénération et contrôle aplati** (AC 8) ; gardes (AC 9).

## Dev Notes

### Ce que la story touche

| zone | fichiers |
|---|---|
| vocabulaire | `docs/i18n-glossaire.md` ; `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` (deux clés, cinq cellules) |
| manuels | `docs/manual/fr/user-manual.tex`, `admin-manual.tex`, `marketing-brochure.tex` + les trois PDF |
| README | `README.md` |

Aucun code applicatif.

### Faits établis à la spécification, et non supposés

- **L'inventaire de l'AC 3** : un grep exécuté sur la branche le 2026-09-15 (douze sites — la première
  rédaction en annonçait onze, avec un motif qui ne pouvait pas trouver `README.md:29`), puis une
  recherche par le concept sur les PDF aplatis en passe 4 (cinq de plus). La parente en citait trois.
- **Les sources de l'alignement** : la règle 3 pour les catalogues, l'arbitrage du soir pour le manuel,
  une décision de la story pour la brochure et le README (AC 1).
- **L'encadré de l'AC 4** a été relu en entier (`user-manual.tex:497-511`) : la parente n'en corrigeait
  que deux énoncés.
- **La suppression d'une facture validée emporte son écriture** : appel `crates/kesh-db/src/repositories/invoices.rs:1350`, commentaire `:1339-1340`,
  `crates/kesh-db/src/repositories/journal_entries.rs:973-975`.

### Ce qui ne bouge PAS

- Le nom de la table `audit_log` là où il désigne la table, et les codes d'action.
- `website/` (terme `en-CH` déjà conforme).
- Les manuels `de`, `it`, `en` : vides en v0.1 (`CLAUDE.md` § *Synchroniser TOUTES les docs*).

### Intelligence des stories précédentes

- **25-1c-zero** : le manuel n'était vu d'aucun grep sur `company_id`, parce que LaTeX écrit
  `company\_id` — il a fallu la revue pour le trouver.
- **Epic 24** (`CLAUDE.md` § *Le prompt d'une passe doit NOMMER le manuel*) : le manuel a été pris en
  défaut six fois sous six formes ; c'est en le confrontant au code qu'on trouve ce que le code ne fait
  pas.
- **Cette story, passes 3 et 4** : un inventaire lexical se clôt sur ses propres mots. *Deux ensembles de
  même cardinal sont indétectables au nombre*, et un concept se désigne aussi sans son terme.

### References

- `25-1c-b1-journal-audit-ecran.md` — **l'écran que ces textes décrivent**
- `25-1c-a-journal-audit-route.md` — AC 2 (filtres), AC 6 (refus, dont les clés API), AC 8 et 17 (libellés traduits, vocabulaire), AC 9 et 14 (ni la consultation ni l'export ne sont tracés), AC 11 (plafond), AC 12 (langue du CSV), AC 15-16 (clés et repli sur le code), sites du manuel d'administration
- `25-1c-b-journal-audit-ecran.md` — la fiche parente, passes 1 et 2
- `epic-25-vague1-suite.md` § *Arbitrage du 2026-09-15 (soir)*
- `docs/i18n-glossaire.md` § A, § *Comment s'en servir*, `:180`

[#378]: https://github.com/guycorbaz/kesh/issues/378

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- **2026-09-15** — **Fiche créée par le split de la 25-1c-b** (arbitrage du Project Lead). Les volets
  « vocabulaire » et « ce que l'écran rend faux » viennent de la parente (AC 13-18) ; deux findings de sa
  passe 2 sont appliqués **ici** :
  - **M5** — l'encadré de `user-manual.tex:497-510` porte deux énoncés périmés de plus que ceux que la
    parente corrigeait (« Une seule exception… », « …tant que le logiciel vous laisse le choix ») → les
    quatre tabulés (AC 4) ;
  - **M6** — la règle 3 s'applique aussi au manuel (`user-manual.tex:1591`, `:1603`, `:1739`) → **le
    grep élargi en rend douze** (onze à la première rédaction, cf. passe 3), dont `audit-trail` dans les
    deux manuels et la brochure, et « piste de contrôle » dans le README (AC 3).
  
  Précision ajoutée au passage : la section « Traçabilité » décrit la sémantique des dates (jours UTC au
  filtre, heure locale à l'écran, UTC au CSV), conséquence du finding M2 appliqué dans la b1 (AC 5).

### Passe 3 de `bmad-create-story validate` — une lentille Sonnet, contexte frais

Prompt versionné : `25-1c-b2-validate-prompt-p3.md`. Base : la parente à `0b029e24` (AC 13-18).

**Rendu : 0 CRITICAL, 1 HIGH, 4 MEDIUM, 2 LOW — tous vérifiés au sol et retenus.**

**Après vérification : 1 HIGH, 2 MEDIUM, 4 LOW.** Deux findings sont reclassés de MEDIUM à LOW (M4 et M5
ci-dessous).

- **H1 (HIGH)** — **l'inventaire de l'AC 3, écrit par la création de cette fiche, était faux**, et son
  total juste par coïncidence. Le grep qu'il citait rendait bien 11 lignes, mais pas les mêmes que le
  tableau :
  - il trouvait `user-manual.tex:301` (« pour préserver l'audit-trail »), absent du tableau ;
  - il ne pouvait pas trouver `README.md:29` (« audit log »), présent au tableau.

  Vérifié : `grep -c "audit-trail" user-manual.tex` rend 3 → ligne ajoutée, motif élargi à
  `audit log`, **onze → douze** dans le corps et au Change Log.

  ⛔ *Deux ensembles différents de même cardinal sont indétectables à la relecture du nombre* — l'acquis 2
  de la 25-1b, reproduit par l'orchestrateur.
- **M2** — une troisième clé du catalogue contient « audit » : `bank-accounts-errors-has-transactions`
  (« audit comptable », fr/it/en). Elle n'était ni alignée ni triée → **triée « autre sens »** (la
  révision des comptes), écrite à l'AC 2.
- **M3** — « la règle 3 vaut aussi pour les manuels » excédait le glossaire, qui ne vise que les clés, et
  distingue lui-même le manuel de l'interface (`i18n-glossaire.md:180`) → l'extension est désormais
  écrite comme une **décision de cette story**, avec son motif : le manuel nomme l'écran.
  *(Corrigé à son tour en passe 4 : pour le manuel, c'est un arbitrage.)*
- **M4 → LOW** — insertion dans les glossaires des manuels, où des entrées voisines existent :
  `\item[Journal]` (`user-manual.tex:1754`) et `\item[Journal entry]` (`admin-manual.tex:2189`). Le point
  d'insertion est précisé.
- **M5 → LOW** — les References citaient « AC 15 (clés) » de la 25-1c-a. C'était juste pour les clés
  i18n, mais le refus des clés API, que l'AC 6 de cette fiche affirme, n'avait aucune référence →
  ajouté « AC 6 (refus, dont les clés API) ».
- **L6** — la suppression d'une facture validée emporte encore son écriture (`journal_entries.rs:973-975`,
  #380/#381) → le texte de remplacement de l'encadré ne doit pas être absolu (AC 4).
- **L7** — citation tronquée d'`admin-manual.tex:1786` → citée en entier.

⚠️ **Signal de sévérité** : la parente avait terminé sa passe 2 à MEDIUM, et la première passe de cette
fiche remonte un HIGH. Ce n'est **pas** un défaut de conception d'origine qui résiste : **l'inventaire a
été écrit par la remédiation elle-même**. C'est le motif mesuré du projet, la sévérité se déplace vers ce
qu'on vient d'écrire. La story ne touche que du texte et ne se découpe pas davantage ; la boucle continue.

Remédiation : texte de spec uniquement.

### Passe 4 de `bmad-create-story validate` — une lentille Opus, contexte frais

Prompt versionné : `25-1c-b2-validate-prompt-p4.md`. Base : `git diff f764004d 637c6508`.

**Rendu : 0 CRITICAL, 0 HIGH, 4 MEDIUM, 7 LOW, et 1 point hors périmètre — tous vérifiés au sol et
retenus.** Sévérité maximale **HIGH → MEDIUM** : la boucle converge.

La passe confirme d'abord la correction du HIGH de la passe 3 : le grep de la fiche rend **exactement**
les douze lignes du tableau, ensemble contre ensemble.

- **M1** — l'inventaire lexical ne voit pas les phrases qui désignent le journal **sans ses mots** :
  `user-manual.tex:858`, `:1684`, `admin-manual.tex:1088`, `:1143`, `:1935` (« La piste existe… »).
  Vérifié ligne à ligne → **dix-sept sites**, deux procédés, motif élargi pour le rejeu (AC 3, 8).
- **M2** — l'AC 4 se contredisait depuis la passe 3 : « **toute** suppression d'écriture est refusée »
  à côté de l'avertissement sur la suppression de facture ; « deux lignes plus haut » était faux ; et
  `admin-manual.tex:1796` écrit **déjà** la formule absolue interdite. Vérifié (`invoices.rs:1339`) →
  justification réécrite, cinquième énoncé tabulé ; `:1796` **porté à l'arbitrage** (AC 6).
- **M3** — la passe 3 avait attribué à la story l'alignement du manuel, alors que **l'arbitrage du soir
  le nomme** (« dans les catalogues et le manuel ») ; et l'AC 1 disait encore « les AC 2 et 3 sont ces
  occurrences ». → tableau des trois sources à l'AC 1, arbitrage cité.

  ⛔ *Un arbitrage résumé est un arbitrage réécrit* : la leçon de la 25-1c-a, reproduite par la
  remédiation d'un finding qui portait justement sur l'attribution d'une règle.
- **M4** — l'AC 2 exemptait de tout contrôle les clés `audit-log-*` neuves, les plus exposées à une
  forme divergente → exemption remplacée par une vérification.
- **L1** — citation de `:1786` introuvable au grep (`\textbf{consulte}`), et lignes attribuées à `main`
  alors qu'elles sont celles de la branche empilée → graphie exacte et en-tête corrigés.
- **L2** — l'encadré a **cinq** énoncés, bornes `497-511` → corrigé (AC 4, Dev Notes).
- **L3** — « audit-trail » gardé en synonyme contredisait l'AC 2 → « (angl. audit log) », reste attendu
  écrit à l'AC 8.
- **L4** — `nav-audit-log` cité comme précédent d'une ligne de glossaire, alors que la clé naît dans la
  PR → précédents déjà livrés cités.
- **L5** — trois cellules sur huit déjà conformes ; `de-CH:936` et les commentaires `# Story 14-2`
  non triés → précisé.
- **L6** — le grep était décrit comme « écartant » des occurrences qu'il ne peut pas rendre → corrigé.
- **L7** — le manuel ne disait rien des cas où l'écran rend une liste vide ou un refus sans explication
  (action en égalité exacte, identifiant sans type, plafond de 10 000, « société indéterminée ») →
  ajoutés à l'AC 5, avec le critère.
- **Hors périmètre** — `admin-manual.tex:1778` annonce « 5 rôles » (il y en a trois, `user.rs:20-27` et
  `:1345`) → **porté à l'arbitrage** avec `:1796` (AC 6).

Remédiation : texte de spec uniquement.

### Passe 5 de `bmad-create-story validate` — une lentille Sonnet, contexte frais

Prompt versionné : `25-1c-b2-validate-prompt-p5.md`. Base : `git diff 637c6508 c5d14f32`.

**Rendu : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 1 LOW — vérifiés au sol et retenus. Après reclassement : 1 MEDIUM,
2 LOW.** Les trois portent sur la remédiation de la passe 4.

- **M1** — le motif élargi de l'AC 8, rejoué sur les sources actuelles, rend **18 lignes**, et l'une d'elles
  n'est ni au tableau ni au tri : `marketing-brochure.tex:209`, « Une piste pour les fiduciaires », une
  piste commerciale. Au rejeu après implémentation, elle aurait fait échouer à tort le critère « reste
  attendu, et seulement lui ». La recherche par le concept n'avait couvert que les deux manuels. →
  triée « autre sens ».

  La lentille note aussi que `user-manual.tex:1684` ne répond pas au motif. C'est sans conséquence : ce
  site se contrôle par sa phrase (T0).
- **M2 → LOW** — l'introduction de l'AC 4 (« aucun ne reste vrai ») contredisait le cinquième énoncé,
  ajouté par la même passe 4 et qualifié de « vrai, à nuancer ». *Reclassé* : la ligne du tableau prescrivait
  déjà la nuance, et seule la phrase d'introduction se contredisait. → « quatre faux ou périmés, le premier
  vrai mais incomplet ».
- **L3** — « six cellules » : le tableau de l'AC 2 compte 3 + 2 = **cinq** cellules à modifier →
  corrigé.

La passe confirme par ailleurs, ensemble contre ensemble :
- les douze sites lexicaux et les cinq sites « concept » ;
- les sept tris « autre sens » de la passe 4 (huit avec `marketing-brochure.tex:209`) ;
- les bornes de l'encadré ;
- les cellules de l'AC 2 et les deux défauts voisins ;
- la citation de l'arbitrage, les précédents de glossaire, et les énoncés de l'AC 5 confrontés au contrat
  de la 25-1c-a et de la b1.

Remédiation : texte de spec uniquement.

### Passe 6 CIBLÉE de `bmad-create-story validate` — une lentille Opus, contexte frais

Prompt versionné : `25-1c-b2-validate-prompt-p6.md`. Base : `git diff c5d14f32` (remédiation de la passe 5).

**Rendu : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 3 LOW, plus 2 remarques hors périmètre — vérifiés au sol et
retenus.**

⛔ **Les deux MEDIUM sont des défauts de PROPAGATION de l'orchestrateur** : la passe 5 a corrigé une phrase
sans corriger celle qui disait la même chose ailleurs. C'est la faute que la § *Propagation post-patch*
du `CLAUDE.md` décrit, commise pendant la boucle qui la cite.

- **M1** — l'introduction de l'AC 4, réécrite en passe 5 (« le premier est vrai mais incomplet — il se
  nuance »), contredisait la conclusion restée intacte (« l'encadré se réécrit **ou disparaît** […] s'il
  reste quelque chose à dire »). Un encadré qui disparaît emporte la phrase vraie qu'on vient d'ordonner
  de garder. → conclusion réécrite : le premier énoncé nuancé est **conservé**.
- **M2** — « six cellules » corrigé aux Dev Notes en passe 5, mais l'AC 2 disait « **trois** cellules
  seulement » (`git log -S` : phrase de la passe 4, qui déformait « trois sur huit conformes ») →
  « cinq cellules — trois sur huit sont déjà conformes ».
- **L3** — le « reste attendu » de l'AC 8 renvoyait aux huit tris, alors que le motif n'en rend que deux ;
  une comparaison exacte aurait attendu six sites introuvables → reste écrit **en entier**.
- **L4** — la note de la passe 5 écrivait `:209` sans fichier, et attribuait à la passe 4 une recherche
  « limitée aux manuels » que son prompt contredit → « le mot « piste » seul n'avait été cherché que dans
  les manuels ».
- **L5** — « les sept tris » sans périmètre → « sept de la passe 4, huit avec `:209` ».
- **Hors périmètre, intégrés** :
  - le PDF d'administration coupe des mots en fin de ligne, d'où un faux négatif possible sur « nouvelles
    phrases présentes » → recollage des césures prescrit à l'AC 8 ;
  - **#380 est fermée** (le 2026-09-09) → l'AC 4 ne la dit plus « en suivi ».

Au passage, l'orchestrateur a constaté que le motif de l'AC 8 était prescrit en `grep -oiE` avec contexte
`{0,60}`, forme qu'`ugrep` refuse ; il est désormais prescrit en Python.

La passe confirme par ailleurs le rejeu de l'AC 8 **sur les PDF aplatis** : aucun site n'échappe au
tableau, et les occurrences en plus viennent du sommaire et des en-têtes.

Remédiation : texte de spec uniquement.

### Passe 7 CIBLÉE de `bmad-create-story validate` — une lentille Sonnet, contexte frais — **BOUCLE CLOSE**

Prompt versionné : `25-1c-b2-validate-prompt-p7.md`. Base : `git diff 06fda3b0` (remédiation de la passe 6).

**Rendu : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 0 LOW.** Les quatre axes sont déclarés exercés, avec leurs
preuves :
- **propagation de chaque phrase modifiée** : AC 4 (introduction, conclusion, tableau, T3), AC 2 et Dev
  Notes (3 ✗ + 2 ✗ = 5, `git log -S` sur l'ancienne phrase), AC 8 et tris de l'AC 3 (8 − 2 = 6), note sur
  la brochure ;
- **le rejeu de l'AC 8 exécuté en Python** sur les trois PDF aplatis et le README, avec et sans recollage
  des césures : décomptes identiques, et seuls les deux sites nommés du reste attendu y apparaissent ;
- **l'état des issues**, confirmé par `gh issue view` et par le commentaire de
  `journal_entries.rs:965,978` ;
- **le Change Log de la passe 6**, recompté.

**Vérification de l'orchestrateur** : l'état de #380 (fermée le 2026-09-09) et de #381 (ouverte) avait
été relevé par lui-même avant la remédiation (`gh issue view`).

**Noté, non appliqué** (LOW latent, sans effet aujourd'hui) : le recollage `(\w)- (\w)` joint aussi les
**jonctions de colonnes** des tableaux du PDF d'administration (« para- Admin »), et pas seulement les
césures. Aucune ne croise le motif. À garder en tête si le rejeu de l'AC 8 rend un jour une occurrence
inattendue.

**Critère de clôture** : 0 finding au-dessus de LOW, et la dernière remédiation (passe 6) ne touche
aucune ligne de code, pas plus que la story, qui n'est que texte.

**Trend de la boucle** (la parente pour les passes 1-2, cette fiche ensuite) :

| passe | modèle | rendu (après reclassement) |
|---|---|---|
| 1 | Sonnet + Haiku | 2 M, 2 L |
| 2 | Opus | 6 M, 7 L → **split** |
| 3 | Sonnet | 1 H, 2 M, 4 L |
| 4 | Opus | 4 M, 7 L |
| 5 | Sonnet | 1 M, 2 L |
| 6 | Opus, ciblée | 2 M, 3 L |
| 7 | Sonnet, ciblée | **0** |

**Ce que la boucle a appris**, et qui vaut au-delà de la story :
- **un inventaire lexical se clôt sur ses propres mots** (passes 3 à 5 : 11 → 12 → 17 sites, puis un
  dix-huitième au tri) ;
- **deux ensembles de même cardinal sont indétectables au nombre** (passe 3) ;
- **la propagation se perd pendant la boucle même qui la cite** (passe 6 : deux phrases jumelles
  laissées intactes par l'orchestrateur).

### Réouverture du 2026-09-15 (soir) — la conception de la 25-1c-a a changé

Trois arbitrages du Project Lead et la réponse *« corrige »*, rendus après la clôture de la boucle, changent ce que la section
« Traçabilité » et le manuel d'administration doivent dire :

| arbitrage | ce qui change ici |
|---|---|
| filtre strict — *« le cas réel n'existe pas »* | la mention « société indéterminée » sort de l'AC 5 |
| l'export n'est pas inscrit au journal — *« non »* | AC 6 : le manuel dit que ni la consultation ni l'export ne sont tracés, au lieu de « l'export est tracé » |
| actions et types traduits, écran et CSV, langue de l'interface | AC 5 : filtres en listes de libellés, langue, repli sur le code d'une action historique ; AC 2 : la clé `audit-log-entity-audit-log` n'existe plus |
| *« corrige »* | AC 6 : `admin-manual.tex:1796` et `:1778` corrigés dans cette story (T4) |

Les références à la 25-1c-a sont alignées sur sa numérotation réécrite (AC 8, 9, 11, 12, 14-17).
Le tableau d'en-tête et les arbitrages (ajout « au soir ») sont mis à jour dans le même geste.
**Revalidation requise**, ciblée sur ces changements.

### Revalidation R1 CIBLÉE — une lentille Opus, contexte frais

Prompt versionné : `25-1c-b2-validate-prompt-r1.md`. Base : `git diff 5760bab7 90a427bf`.

**Rendu : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 6 LOW (dont 2 hors diff) — tous vérifiés au sol et retenus.**

- **M1** — les réponses *« ok »* (écran traduit), *« non, maintenant »* et *« corrige »* étaient attribuées
  à l'epic, qui ne les portait pas : leur **seule trace écrite était la story qu'elles élargissent**.
  Vérifié (`grep` sur l'epic). → les trois questions et leurs réponses inscrites à l'epic, § *fin de
  soirée* ; paragraphe périmé « société indéterminée » barré ; « 81 codes » → 82.
- **M2** — la vérification des catalogues n'était plus exécutable : le grep `audit` porte aussi sur les
  **noms** de clés, et les 123 clés `audit-log-*` × 4 l'auraient noyé ; et un grep sur « audit » ne voit
  pas « Protokoll » ou « journal » seul. → grep hérité restreint aux autres clés, procédé Python **sur la
  valeur**, liste des clés d'écran de la b1 complétée, « à la spécification ».
- **L1** — une phrase attribuait à la 25-1c-a une prose sur « la piste » que sa réécriture a supprimée
  (`grep -c "la piste"` = 0) → retirée.
- **L2** — `admin-manual.tex:1797` (« suppressions antérieures au gel ») devient trompeuse une fois `:1796`
  nuancée → ajoutée à l'AC 6 et à T4.
- **L3** — une action historique s'affiche en code mais **n'est pas choisissable** dans la liste
  « Action » → le manuel le dira (AC 5).
- **L4** — la fiche ne se disait pas réouverte ; « trois arbitrages » pour quatre lignes → bandeau et
  tableau, Change Log précisé.
- **L5** — citation « seul site du dépôt » attribuée à `journal_entries.rs` → `invoices.rs:1339-1340`,
  appel à `:1350`.
- **L6** (epic) — paragraphe contradictoire et titre « deux des trois » → corrigés avec M1.

**Remarque laissée telle quelle** : `admin-manual.tex:1786` décrit la colonne `company_id` (« vide quand
elle est indéterminable ») ; c'est une description de la **table**, qui reste vraie, et non de l'écran.

Remédiation : texte de spec et de planning uniquement.

### Revalidation R2 CIBLÉE — une lentille Sonnet, contexte frais

Prompt versionné : `25-1c-b2-validate-prompt-r2.md`. Base : `git diff 2efbc76c 6cda8b89` et la section
*fin de soirée* de l'epic (commit `32ebe675`).

**Rendu : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 3 LOW — vérifiés au sol et retenus.** Sévérité maximale inchangée
(MEDIUM), et **les deux MEDIUM sont des propagations oubliées de la remédiation R1** — le même mode d'échec
que la passe 6.

- **M1** — l'epic comptait encore **82** codes d'action, alors que la R1 de la 25-1c-a en a établi **92** ;
  le commit `6cda8b89` se disait « comptes 133/92 propagés » sans toucher l'epic. → 92, avec la généalogie
  des deux comptes faux.
- **M2** — la R1 a ajouté `:1797` aux défauts voisins sans propager « **deux** défauts » (trois sites) ni la
  table de l'AC 6. → « trois », ligne ajoutée à la table.
- **L3** — le procédé Python ne lisait pas une valeur Fluent **multiligne** (exécuté sur un patron du dépôt)
  → rattachement des lignes de continuation.
- **L4** — l'exclusion des clés `audit-log-*` du grep hérité n'était écrite comme aucune commande →
  pipeline écrit.
- **L5** — `invoices.rs` et `journal_entries.rs` existent en **deux** exemplaires (routes et repositories) →
  chemin complet du repository.

**Noté sans suite** : les bases de la R1 (`5760bab7`, `90a427bf`) sont devenues des objets pendants au
rebase ; **le fichier** de la fiche y est identique à celui de `37596132` et `2efbc76c` *(« leurs arbres »,
d'abord écrit, était faux — corrigé en R3)*.

Remédiation : texte de spec et de planning uniquement. **Revalidation R3 ciblée requise** (MEDIUM).

### Revalidation R3 CIBLÉE — une lentille Opus, contexte frais

Prompt versionné : `25-1c-b2-validate-prompt-r3.md`. Base : `git diff 5dfbafc2 0bbefc51` (fiche et epic).
Sondes : pipeline exécuté sur les `.ftl` réels et sur une copie jetable enrichie de clés `audit-log-*`,
extraction positionnelle indépendante (92 actions confirmées), objets git et reflog.

**Rendu : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 3 LOW — vérifiés et retenus.** Les quatre viennent de la remédiation R2.

- **M1** — la R2 rangeait `:1797` sous la réponse *« corrige »*, alors que la question ne portait que sur
  `:1796` et `:1778` (epic, § *fin de soirée*) : *un arbitrage résumé est un arbitrage réécrit*, troisième
  occurrence dans cette fiche. → `:1797` présentée comme **décision de la story**, avec son motif.
- **L1** — le filtre `grep -vE ':[0-9]+:(nav-)?audit-log'` n'ôte que la ligne de clé : continuations,
  attributs, variantes et en-têtes de blocs neufs sortent encore (exécuté sur copie jetable) → écrit.
- **L2** — des renvois restaient sans chemin complet, dont un préfixe `crates/` manquant → **trois** sites
  qualifiés *(« deux » d'abord écrit ici : deux symptômes, trois sites — recompté en R4)*.
- **L3** — la note « arbres identiques » était fausse (seul le fichier l'est), et **les bases citées par la R2
  elle-même** (`2efbc76c`, `6cda8b89`, `32ebe675`) ont disparu des branches au rebase suivant : équivalents
  actuels `dd3cedab`, `5dfbafc2`, `7f8b0288`. Les prompts versionnés restent tels qu'envoyés. ⚠️ **Leçon de
  méthode** : une base de revue citée par hash sur une branche empilée ne survit pas au rebase — citer aussi
  le **message** du commit, qui, lui, survit.

La passe confirme par ailleurs : les trois lignes du manuel et leurs citations, les deux repositories aux lignes
dites, la généalogie « 81, puis 82, puis 92 », et les décomptes du Change Log R2.

Remédiation : texte de spec uniquement. **Revalidation R4 ciblée requise** (MEDIUM).

### Revalidation R4 CIBLÉE — une lentille Sonnet, contexte frais — **BOUCLE CLOSE**

Prompt versionné : `25-1c-b2-validate-prompt-r4.md`. Base : `git diff 44d638f9 8e729449` — le commit
« docs(25-1c-b2): revalidation R3 ciblée ». Sondes : pipeline rejoué sur copie jetable (valeur multiligne,
attribut, variante, en-tête de bloc), objets git et reflog.

**Rendu : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 2 LOW — vérifiés et appliqués.** Les quatre axes sont déclarés exercés.

- **L1** — un renvoi `invoices.rs:1339-1340` restait sans chemin, dans la phrase même que la R3 qualifiait →
  qualifié.
- **L2** — le Change Log R3 disait « deux renvois qualifiés » pour trois sites → recompté.

La passe confirme : `:1797` n'est plus rattachée à *« corrige »* nulle part ; la phrase sur le filtre du grep
est exacte (exécutée) ; la note « le fichier est identique » et les équivalents de hash sont exacts.

**Critère de clôture** : aucun finding au-dessus de LOW, et la dernière remédiation ne touche que du texte de
spec — la story, d'ailleurs, ne touche aucun code applicatif.

**Trend de la boucle rouverte** (après la clôture en 7 passes de la version d'avant les arbitrages du soir) :

| passe | modèle | rendu |
|---|---|---|
| R1 | Opus, ciblée | 2 M, 6 L |
| R2 | Sonnet, ciblée | 2 M, 3 L |
| R3 | Opus, ciblée | 1 M, 3 L |
| R4 | Sonnet, ciblée | **2 L** |

**Ce que la boucle rouverte apprend** : aucun des MEDIUM de R1 à R3 ne portait sur la conception. Tous venaient
de la remédiation précédente, sous **deux** formes seulement : une **propagation oubliée** (un nombre corrigé à
un site, laissé à son jumeau), et un **arbitrage résumé** (une réponse du Project Lead étendue à ce qu'elle ne
nommait pas). La seconde forme est la plus coûteuse : elle se lit comme juste.

# Story 25.1c-b2 : Ce que l'écran du journal d'audit rend faux — manuels, README, vocabulaire

Status: review

⛔ **Historique de la fiche** : validée en 7 passes le 2026-09-15, rouverte, revalidée le 2026-09-16 ; restée
dix jours sur une branche locale jamais poussée ; la revalidation R5 « dérive » (2026-09-26) l'a trouvée
périmée — d'autres stories avaient réécrit une partie des textes visés, et une facture validée se
**dévalide** désormais au lieu de se supprimer. **Réécrite en T0 le 2026-09-26** contre le texte de `main`,
après l'implémentation de la 25-1c-b1, puis revalidée (R6, R7 — Change Log). Les lignes citées sont celles
du 2026-09-26 ; **chaque site se retrouve par sa phrase**.

| | objet | état au 2026-09-26 |
|---|---|---|
| 25-1c-zero | la colonne `audit_log.company_id` | mergée (PR #437) |
| 25-1c-a | la route de consultation, son vocabulaire traduit et son export CSV | mergée (PR #439) |
| 25-1c-b1 | l'écran | implémentée, revue close, même branche |
| **25-1c-b2** *(celle-ci)* | **les textes : manuels, README, CHANGELOG, vocabulaire « journal d'audit »** | fiche réécrite |

⛔ **Livrée dans la MÊME PR que la 25-1c-b1**, qui porte `closes #378` : les textes décrivent l'écran **tel
qu'il est construit**.

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

⛔ **RÉÉCRITS LE 2026-09-26 (T0), contre le texte de `main` à `0e4c2682` + la 25-1c-b1** — la version
validée en 2026-09-16 décrivait des textes que d'autres stories ont réécrits depuis, et un mécanisme
(« la suppression d'une facture validée emporte son écriture ») qui n'existe plus : une facture validée
se **dévalide** (`invoices.rs:1293`, `InvoiceMustBeUnvalidatedFirst` ; action `invoice.unvalidated`).
**Les numéros de ligne sont ceux du 2026-09-26 ; chaque site se retrouve par sa PHRASE citée.**

### Volet A — le vocabulaire « journal d'audit »

**1. Le glossaire** — `docs/i18n-glossaire.md`, **partie A**, une ligne :
« journal d'audit » | `Audit-Protokoll` | `registro di audit` | `audit log` | précédents **déjà livrés** :
`export-global-content-excludes` (réécrite par la 25-5-a, **conforme dans les quatre locales**) et
`fiscal-year-reopen-confirmation-body` en allemand (« Audit-Protokoll »). ⚠️ **Ne pas citer** `nav-audit-log`
ni les clés d'écran de la 25-1c-b1 : elles naissent dans la même PR (le glossaire pose qu'une clé qu'on
écrit n'est pas un précédent).

**2. Les catalogues** — **une seule clé** s'écarte encore du terme, sur **trois** cellules (la 25-5-a a
aligné l'autre, `export-global-content-excludes`) :

| clé | fr | de | it | en |
|---|---|---|---|---|
| `fiscal-year-reopen-confirmation-body` | « piste d’audit » ✗ | « Audit-Protokoll » ✓ | « pista di audit » ✗ | « audit trail » ✗ |

⇒ fr « journal d’audit » (apostrophe **typographique** `’`, comme le reste de la valeur), it « registro di
audit » (« sarà conservato nel registro di audit »), en « audit log ». La règle 3 du glossaire l'impose :
la ligne de l'AC 1 ne s'écrit pas sans cet alignement.

⛔ **Et son repli Svelte** (revalidation R6) : `frontend/src/routes/(app)/settings/fiscal-years/+page.svelte:554`
porte « … sera conservé dans la piste d'audit. » → « … dans le journal d'audit. », **mot pour mot** le FTL
fr-CH (à l'apostrophe près : le repli emploie l'apostrophe droite `'` là où le FTL a `’` — la garde
`i18n-un-repli-par-cle` dit si l'écart compte). Le catalogue gagne sur le repli, le défaut est latent — mais
la règle 3 vise « toutes ses occurrences déjà livrées ».

**Contrôle**, exécutable, **sur la VALEUR seule** (Python) : pour chaque `.ftl`, chaque clé dont la valeur
— lignes de continuation rattachées — contient `audit|protokoll|registro|journal|piste|trail` (sans casse)
est triée à la main. **Triés, à laisser** : `bank-accounts-errors-has-transactions` (« audit comptable » :
la **révision**, autre sens, quatre langues) ; les clés `audit-log-*` du vocabulaire de la 25-1c-a (codes
et libellés d'actions : « écriture », etc.) ; les clés d'écran de la b1, **déjà conformes** (vérifié :
« Journal d'audit » / « Audit-Protokoll » / « Registro di audit » / « Audit log »). ⚠️ Le motif `journal`
rend une quarantaine de clés « journal comptable », et `protokoll` des mots allemands d'un autre sens
(`protokolliertem Grund`, `Serverprotokolle`) : **tri à la main**, attendu.

**3. Les manuels, la brochure et le README** — inventaire du 2026-09-26, réunion de deux procédés :
**lexical** (`grep -niE "piste d.audit|audit-trail|audit trail|audit log|piste de contr|trace d.audit|dans l.audit\b"`
sur `docs/manual/fr/*.tex` et `README.md`) et **par le concept** (une phrase qui désigne le journal sans
ces mots) :

| fichier | ligne | texte actuel | devient |
|---|---|---|---|
| `user-manual.tex` | `:301` | « (pour préserver l'audit-trail) » | « (pour préserver le journal d'audit) » |
| `user-manual.tex` | `:888` | « nouvelle trace d'audit » | « nouvelle entrée au journal d'audit » |
| `user-manual.tex` | `:1111` | « journalisées dans l'audit-trail » | « inscrites au journal d'audit » |
| `user-manual.tex` | `:1789` | `\section{Traçabilité (audit-trail)}` | `\section{Traçabilité (journal d'audit)}` |
| `user-manual.tex` | `:1791` | « une \textbf{piste d'audit} (\texttt{audit\_log}) » | « un \textbf{journal d'audit} (\texttt{audit\_log}) » — accords suivants ajustés |
| `user-manual.tex` | `:1803-1812` | « Cette piste d'audit est » … « Elle \textbf{contribue} » … « elle est \textbf{enregistrée} » | « Ce journal d'audit est » … « Il contribue » … « il est enregistré » — **tous** les accords du paragraphe |
| `user-manual.tex` | `:1889` | « L'\texttt{audit\_log} garde toujours la trace » *(concept)* | « Le journal d'audit garde toujours la trace » |
| `user-manual.tex` | `:1944` | `\item[Audit-trail (piste d'audit)]` — glossaire | `\item[Journal d'audit (angl. audit log)]`, **déplacé à la lettre J**, après `\item[Journal]` (le journal des écritures) — la définition rend la différence lisible ; l'avertissement de couverture qui suit est **conservé** |
| `admin-manual.tex` | `:1088` | « La demande reste tracée dans l'audit » | « … dans le journal d'audit » |
| `admin-manual.tex` | `:1141` | « l'audit-trail (\texttt{invoice.emailed}, …) » | « le journal d'audit (…) » |
| `admin-manual.tex` | `:1143` | « laisse sa propre trace d'audit » | « laisse sa propre entrée au journal d'audit » |
| `admin-manual.tex` | `:1589` | « les conséquences sur la piste de contrôle » | « … sur le journal d'audit » |
| `admin-manual.tex` | `:1760` | « La piste de contrôle se lit dans l'interface web » | « Le journal d'audit se lit dans l'interface web » |
| `admin-manual.tex` | `:1766` | `\item \textbf{Audit} : les mutations […] tracées dans \texttt{audit\_log}` | `\item \textbf{Journal d'audit} : …` (le nom de table reste) |
| `admin-manual.tex` | `:1784` | `\subsection{Audit-trail (audit\_log)}` | `\subsection{Journal d'audit (audit\_log)}` |
| `admin-manual.tex` | `:1816` | « les entrées d'audit \textbf{déjà présentes chez vous} » | « les entrées du journal d'audit déjà présentes … » |
| `admin-manual.tex` | `:1817` | « la piste de contrôle mêlerait deux comptabilités » | « le journal d'audit mêlerait … » |
| `admin-manual.tex` | `:1946` | « la piste de contrôle voyage avec l'export » *(la ligne cite aussi `audit\_log`, la table : lui reste)* | « le journal d'audit voyage avec l'export » |
| `admin-manual.tex` | `:1969` | « \texttt{audit\_log} en conserve la trace » *(concept)* | « le journal d'audit en conserve la trace » |
| `admin-manual.tex` | `:2197` | `\item[Audit-trail]` — glossaire | `\item[Journal d'audit (angl. audit log)]`, **déplacé à la lettre J**, avant `\item[Journal entry]` ; avertissement conservé |
| `marketing-brochure.tex` | `:319` | « Audit-trail centralisé pour la responsabilité fiduciaire. » | « Journal d'audit centralisé … » |
| `README.md` | `:29` | « audit log » (liste des fonctionnalités, en français) | « journal d'audit consultable à l'écran » |
| `README.md` | `:220` | « la **piste de contrôle survit désormais à l'import d'une sauvegarde** » — ligne **v0.12.1, non publiée** | « le **journal d'audit survit désormais …** » |

**Déjà conformes, à laisser** : `user-manual.tex:474`, `:514`, `:543`, `:593`, `:1039`, `:1085`, `:1159`,
`:1427`, `:1712`, `:1721` ; `admin-manual.tex:1066`, `:1613`, `:1810` ; `marketing-brochure.tex:139`.
**Autre sens, à laisser** : `admin-manual.tex:159`, `:1242`, `:2082` (`audit\_log` y nomme la
**table**, non le concept) ; `README.md:219` (« audit par trois experts ») ; `user-manual.tex:624` (« une
piste de correction lisible ») ; `marketing-brochure.tex:209` (« Une piste pour les fiduciaires », piste
commerciale). ⚠️ **L'historique publié ne se réécrit pas** :
la ligne **v0.12.0** du README (`:219`, dont « n'est consultable par aucun écran ») et la section
`[0.12.0]` du CHANGELOG ne se touchent pas. La ligne **v0.12.1**, elle, n'est pas publiée : elle se met à
jour (AC 3, AC 7).
`website/` (anglais, « audit log » déjà) ne se touche pas.

⛔ **Le nom technique `audit\_log`** ne change pas là où il désigne la table ou le champ.

### Volet B — ce que l'écran rend faux

**4. Le manuel utilisateur, l'encadré** `\begin{keshwarning}[title=… Ce que le logiciel ne fait pas
encore à votre place]` (`user-manual.tex:511-528`, juste avant `\subsection{Numérotation}`). Il porte
aujourd'hui **cinq** énoncés, dont **trois sont périmés** :

| énoncé actuel | état |
|---|---|
| « La contre-passation est désormais imposée […] : la modification et la suppression d'une écriture sont refusées » | **vrai**, à conserver — avec la seule voie qui fait disparaître une écriture : la **dévalidation** d'une facture (renvoi à « Supprimer ou dévalider une facture ») |
| « Ce qui manque encore, c'est l'écran de consultation du journal d'audit […] par l'interface de programmation seulement […] La page […] reste à venir » | **faux** : l'écran existe (25-1c-b1) |
| « Un verrou plus fin que l'exercice existe, lui : le verrou de période […] » | **vrai**, à conserver |
| « Une seule exception, déjà en vigueur : une écriture déjà contre-passée ne peut plus être supprimée » | **périmé** : **aucune** écriture ne se supprime plus |
| « préférez la contre-passation […], tant que le logiciel vous laisse le choix » | **périmé** : il ne laisse plus le choix |

⇒ L'encadré devient une **note** (`keshnote`), qui n'annonce plus de fonction manquante : contre-passation
imposée ; **en usage courant**, la dévalidation d'une facture est la seule voie qui fait disparaître une
écriture, **journalisée sous son propre nom** — ⚠️ **hors restauration d'une sauvegarde**, qui remet les
livres dans l'état de l'archive (`backup.rs`, import d'installation), et hors réinitialisation des données de
démonstration, refusée après la finalisation (revalidation R6 : `delete_in_tx` n'a qu'un appelant qui
supprime, `invoices.rs` — la dévalidation ; mais `backup.rs:457` et `kesh-seed` effacent des tables) ;
l'historique des corrections **consultable à l'écran** — *Administration → Journal d'audit*, renvoi à la
section « Traçabilité », qui reçoit pour cela un `\label{sec:tracabilite}` — et par l'API ; le verrou de période conservé tel quel. ⛔ **Aucune formule
absolue** (« en aucun cas », « seule » sans réserve) sur la disparition d'une écriture.

⚠️ **Voisin, dans la même section** (`user-manual.tex:536`, sous-section « Numérotation ») : « Si vous
supprimez la dernière écriture d'un exercice, la suivante prendra le numéro d'après » — une écriture ne se
supprime plus → « Si la dernière écriture d'un exercice disparaît (par la dévalidation de sa facture), la
suivante… ».

**5. Le manuel utilisateur, section « Traçabilité »** — la fin de son `keshnote` (`:1812-1816`) :
*« n'est pas encore consultable via un écran dédié dans l'interface (page de consultation prévue pour une
version ultérieure) »* → une **sous-section « Consulter le journal d'audit »** qui décrit l'écran **tel que la
25-1c-b1 le construit** — critère : **chaque cas où l'écran rend une liste vide, un refus ou une valeur
inattendue se dit** :

- menu **Administration → Journal d'audit**, réservé au **Comptable** et à l'**Administrateur** ; refusé
  aux clés API ;
- les filtres : période, type d'entité, numéro d'entité, action ; ⛔ **les dates filtrent sur des jours
  UTC**, le tableau affiche l'heure **locale** — une opération faite peu après minuit, heure suisse,
  appartient au jour UTC **précédent** ; une date de début postérieure à la date de fin est refusée par
  l'écran ;
- **le numéro d'entité exige un type** : le champ reste inactif sans type, et se vide quand on change de
  type ; un numéro nul, négatif ou non entier est ignoré ; une entrée sans entité précise affiche « — » ;
- **la langue** : actions et types d'entité traduits dans la langue de l'installation, à l'écran comme dans
  le CSV ;
- **une action historique**, absente du vocabulaire, s'affiche sous son **code** et n'est pas proposée
  dans la liste « Action » — on la retrouve par le type et le numéro d'entité ;
- le détail déplié (JSON), la pagination ;
- **l'export CSV** porte les filtres affichés, en UTC, **plafonné à 10 000 lignes** — au-delà, un message
  demande d'affiner ; ⚠️ une plage inversée, à l'export, est refusée par le serveur, avec un message qui
  n'est pas encore traduit (#469) ;
- ⚠️ ni la consultation ni l'export ne s'inscrivent eux-mêmes au journal.

La phrase sur le **centre de notifications** (« prévu mais pas encore disponible ») reste vraie et se
conserve. Les **manques connus** de la section (séquence d'installation #434, gestes de session #435,
**toutes deux ouvertes**) restent.

**6. Le manuel d'administration** :

- **item « Champs »** de la sous-section du journal (`admin-manual.tex:1790`) : *« L'\textbf{écran}, lui,
  reste à venir (issue \#378). »* → l'écran existe (*Administration → Journal d'audit*, Comptable et
  Administrateur, refusé aux clés API) ; la suite de la phrase (« ne filtre que votre société, et ne
  s'inscrit pas elle-même au journal ») se conserve ;
- `:1782` — « Les **5** rôles standards » → **trois**, en accord avec `:1345` (`entities/user.rs`) ;
- `:1803` — « Il conserve aussi les modifications et suppressions **antérieures au gel** » : à compléter
  par les **dévalidations** de factures, postérieures au gel et journalisées (`invoice.unvalidated`) — sans
  quoi la phrase laisse croire qu'aucune disparition postérieure n'est tracée ;
- `:1802` — « ⚠️ \textbf{Une seule voie fait encore disparaître une écriture} » : même réserve que l'AC 4 —
  **en usage courant**, hors restauration d'une sauvegarde (la sous-section « Restaurer » et l'encadré des
  réserves le disent déjà) ; le reste du paragraphe (dévalidation, huit motifs) se conserve.

**7. Le README et le CHANGELOG** :

- `README.md:220` (ligne v0.12.1 de la feuille de route) : « **Restent ouverts** : l'**écran** de
  consultation ([#378]) — la piste de contrôle se lit désormais par l'API, […] mais aucune page ne la rend
  lisible depuis l'application — » → l'écran est **livré** ; #434 et #435 restent ouverts ;
- `CHANGELOG.md`, section `[0.12.1] — Non publié`, *Added* : **« Consulter le journal d'audit à
  l'écran »** — l'entrée de la 25-1c-a (`[0.12.0]`, « La piste de contrôle se lit enfin ») disait la route
  seule ; ⛔ ne pas la réécrire.

### Volet C — le contrôle

**8. Régénération et vérification** :

- `make fr` dans `docs/manual/`, PDF commités ;
- **PDF aplatis vérifiés** (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`, **césures recollées**,
  `re.sub(r"(\w)- (\w)", r"\1\2", t)`) : les anciennes phrases absentes, les nouvelles présentes ;
- **inventaire REJOUÉ** sur les PDF aplatis et le README, motif, sans casse :
  `piste|trace d.audit|dans l.audit\b|audit-?trail|audit log` — **reste attendu, et seulement lui** : les
  deux « (angl. audit log) » des glossaires ; `user-manual.tex:624` (« piste de correction ») ;
  `marketing-brochure.tex:209` (« Une piste pour les fiduciaires ») ; dans le README, la ligne **v0.12.0**
  (historique publié). On compare des **sites**, pas des nombres (un site peut apparaître au sommaire).

**9. Les gardes** : `cargo test -p kesh-i18n` (parité) ; `npm run test:unit` — la clé de l'AC 2 change de
**valeur**, pas de nom : **aucun compteur ne doit bouger** ; s'il bouge, l'écrire.

## Tasks / Subtasks

- [x] **T0 — Réécriture de la fiche** contre le texte actuel, **après la 25-1c-b1** *(fait le 2026-09-26,
      cette version)* ; puis revalidation.
- [x] **T1 — Glossaire et catalogue** (AC 1, 2).
- [x] **T2 — Vocabulaire des manuels, de la brochure et du README** (AC 3).
- [x] **T3 — Encadré et section « Traçabilité » du manuel utilisateur** (AC 4, 5) — chaque affirmation
      neuve contrôlée **contre le code de la b1**.
- [x] **T4 — Manuel d'administration, README, CHANGELOG** (AC 6, 7).
- [x] **T5 — Régénération, contrôle aplati, inventaire rejoué** (AC 8) ; gardes (AC 9).

## Dev Notes

### Ce que la story touche

| zone | fichiers |
|---|---|
| vocabulaire | `docs/i18n-glossaire.md` ; `crates/kesh-i18n/locales/{fr,it,en}-CH/messages.ftl` (une clé, trois cellules) ; le repli `frontend/src/routes/(app)/settings/fiscal-years/+page.svelte:554` |
| manuels | `docs/manual/fr/user-manual.tex`, `admin-manual.tex`, `marketing-brochure.tex` + les trois PDF |
| README, CHANGELOG | `README.md`, `CHANGELOG.md` |

Aucun code applicatif, hors **un repli Svelte** (AC 2).

### Faits établis à la réécriture (2026-09-26), et non supposés

- **Une facture validée ne se supprime plus**, elle se dévalide (`crates/kesh-db/src/repositories/invoices.rs:1293`,
  `DbError::InvoiceMustBeUnvalidatedFirst`) ; #381 et #440 sont **fermées**. Le manuel d'administration
  (`:1802`) le dit déjà.
- **L'inventaire de l'AC 3** : grep lexical et lecture par le concept exécutés sur `main` le 2026-09-26.
- **`export-global-content-excludes`** a été réécrite par la 25-5-a dans les quatre locales, et y est
  conforme (vérifié : « le journal d'audit » / « das Audit-Protokoll » / « il registro di audit » / « the
  audit log »).
- **#434 et #435 sont ouvertes** (vérifié `gh issue view`).

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

Claude Opus 5.5 — réécriture de la fiche (T0) et implémentation.

### Debug Log References

- La substitution « piste d’audit » → « journal d’audit » a d'abord laissé l'article féminin
  (« dans la journal d’audit ») : corrigé avant tout commit, trouvé en relisant la valeur.
- `admin-manual.tex:1946` porte « la piste de contrôle **VOYAGE** avec l'export », en capitales — la
  phrase citée par la fiche, en minuscules, ne se trouvait pas : relue sur la source, accords compris
  (« il n'est plus seulement conservé »).
- ⛔ **Défaut de la b1 trouvé en relisant le glossaire** : sa section « Registre » prescrit la **2ᵉ personne
  du singulier** en italien ; le sous-titre de l'écran disait « della vostra società » → « della tua
  società ». L'allemand (« Ihres », forme *Sie*) est conforme.

### Completion Notes List

- **Glossaire** : ligne « journal d'audit / Audit-Protokoll / registro di audit / audit log » en partie A,
  avec ses précédents et les formes proscrites.
- **Catalogue** : `fiscal-year-reopen-confirmation-body` alignée en fr, it, en ; **son repli Svelte**
  (`fiscal-years/+page.svelte`) aussi.
- **Vocabulaire** : les 21 sites de l'inventaire de l'AC 3 (dont les accords de la section
  « Traçabilité »), les deux entrées de glossaire des manuels renommées « Journal d'audit (angl. audit log) »
  et déplacées à la lettre J — celle du manuel utilisateur dit la différence avec le *journal* des
  écritures.
- **Encadré** du manuel utilisateur devenu une **note** : contre-passation imposée ; en usage courant, la
  dévalidation comme seule disparition d'une écriture, **hors restauration d'une sauvegarde** ; consultation
  à l'écran ; verrou de période. La phrase « Si vous supprimez la dernière écriture » de la numérotation
  réécrite.
- **Section « Traçabilité »** : `\label{sec:tracabilite}` et sous-section **« Consulter le journal
  d'audit »**, chaque comportement contrôlé contre le code de la b1 (rôles, jours UTC, plage inversée,
  numéro exigeant un type et vidé au changement, action ancienne, « — », export plafonné et plage
  inversée à l'export → #469, rien de journalisé).
- **Manuel d'administration** : « l'écran reste à venir » → existe ; « 5 rôles » → trois ; `:1803`
  complété des dévalidations ; `:1802` nuancé « en usage courant » avec renvoi à la restauration.
- **README** (ligne v0.12.1 : écran livré, #434 / #435 restent ouverts ; liste des fonctionnalités) et
  **CHANGELOG** `[0.12.1]` *Added* « Consulter le journal d'audit à l'écran ». L'historique publié
  (v0.12.0) n'est pas touché.
- **Contrôle aplati** (césures recollées) : 0 `??` ; 8 anciennes phrases absentes, 8 nouvelles présentes ;
  **inventaire rejoué** — reste exactement l'attendu (deux entrées de glossaire, « piste de correction »,
  « Une piste pour les fiduciaires », la ligne v0.12.0 du README).

### Gates *(b1 + b2, sur la même branche)*

- Frontend : `check` 0 erreur (27 avertissements), `lint-i18n-ownership` PASS, `test:unit` **817 / 817**
  (777 + 40 de la b1), build OK ; gardes i18n **inchangées** par la b2 (une valeur de clé et un repli).
- Backend : `cargo test -p kesh-i18n` 29 / 29 ; base remise à zéro, `scripts/test-fast.sh` **2463 / 2463**.
- E2E : `kesh_e2e` reconstruite, montage complet, frontend buildé après le dernier patch, run à
  **13:30 UTC** : **224 passés / 7 échoués / 19 ignorés, zéro régression** — les 7 de KF-029.

### File List

| Fichier | Nature |
|---|---|
| `docs/i18n-glossaire.md` | ligne « journal d'audit » en partie A |
| `crates/kesh-i18n/locales/{fr,it,en}-CH/messages.ftl` | `fiscal-year-reopen-confirmation-body` ; it-CH : sous-titre de l'écran (registre) |
| `frontend/src/routes/(app)/settings/fiscal-years/+page.svelte` | repli |
| `docs/manual/fr/user-manual.tex` / `.pdf` | vocabulaire, encadré, numérotation, « Traçabilité », glossaire |
| `docs/manual/fr/admin-manual.tex` / `.pdf` | vocabulaire, « Champs », rôles, `:1802-1803`, glossaire |
| `docs/manual/fr/marketing-brochure.tex` / `.pdf` | vocabulaire |
| `README.md`, `CHANGELOG.md` | feuille de route v0.12.1, fonctionnalités, *Added* |

## Change Log

- **2026-09-26** — **revue de code P1** (Sonnet, prompt `25-1c-b2-review-prompt-p1.md`) — **1 HIGH, 1 MEDIUM**, tous deux confirmés par grep avant correction. **HIGH** : l'inventaire déclaré « rejoué » ne l'avait pas été sur tout le dépôt — quatre résidus hors de la liste de la fiche : le commentaire d'en-tête de l'écran lui-même (`+page.svelte:5`), `docs/api-external.md:81`, `docs/user-guide/fr/getting-started.md:12` (« audit-trail », forme que le glossaire de cette story proscrit) et `website/roadmap.html:259` (« full audit trail »), ce qui dément le « `website/` déjà conforme » de la fiche. **MEDIUM** : `admin-manual.tex:1946`, « elle part avec les livres » — accord resté sur l'ancien terme féminin, présent dans le PDF. Tout corrigé, PDF d'administration régénéré et contrôlé aplati (phrase corrigée présente, 0 `??`). Grep du symptôme rejoué sur `docs/`, `README.md`, `website/`, `frontend/src`, les FTL et le CHANGELOG : restent le glossaire, la ligne v0.12.0 du README, une généalogie datée de `i18n-keys.test.ts` et l'historique publié du CHANGELOG, tous légitimes. Le rapport déclare ses axes non exercés : gates non réexécutables par la lentille, lecture phrase par phrase de « Traçabilité » faite par échantillonnage.
- **2026-09-26** — **dev** — Implémentée (Opus 5.5) sur la branche de la b1. Glossaire, une clé et son repli, 21 sites de vocabulaire, encadré devenu note (« en usage courant, hors restauration »), sous-section « Consulter le journal d'audit » contrôlée contre le code, manuel d'administration, README, CHANGELOG. **Défaut de la b1 trouvé en route** : registre italien du sous-titre (« vostra » → « tua »). Contrôle aplati et inventaire rejoué conformes. Gates **b1 + b2** : frontend 817/817, backend 2463/2463, E2E 224/7/19 sans régression.
- **2026-09-26** — **revalidation R7 ciblée** — **Une lentille Haiku 4.5**, contexte frais, braquée sur la seule remédiation R6 (`git diff ef575755 5291c693`), prompt `25-1c-b2-validate-prompt-r7-ciblee.md`. **0 finding** : chaque ligne ajoutée vérifiée par `grep -nF` à sa source ; repli Svelte présent ; réserve « hors restauration » fondée contre `backup.rs` et `kesh-seed` ; aucun « seule » sans réserve ; **grep lexical rejoué — 20 lignes, ensemble égal à celui de l'inventaire**. La remédiation ne touchait que la fiche ⇒ **boucle close**, fiche validée. *(Trend : R5 Sonnet 1 CRIT / 4 HIGH → réécriture T0 → R6 Opus 1 HIGH / 4 MED → R7 Haiku 0.)*
- **2026-09-26** — **revalidation R6** — **Une lentille Opus**, passe complète, prompt `25-1c-b2-validate-prompt-r6.md`, axes déclarés (non exercés : fiche de la b1 lue par son code seulement, manuels DE/IT/EN, lecture exhaustive de « trace » / « tracé »). **1 HIGH, 4 MEDIUM, 8 LOW**, tous retenus : ① HIGH — `user-manual.tex:301` (« pour préserver l'audit-trail »), rendu par le grep même de la fiche, **absent de l'inventaire** : c'est le HIGH de la passe 3 qui revient, perdu à la réécriture. ② MEDIUM — `admin-manual.tex:1760` et `:1946` mal classés (« piste de contrôle » y désigne le journal) ; ③ MEDIUM — trois sites non triés au rejeu (`user-manual.tex:624`, `marketing-brochure.tex:209` — tri de la passe 5 perdu —, `README.md:220`, ligne v0.12.1 **non publiée**) ; ④ MEDIUM — « la dévalidation, seule disparition d'une écriture » est **faux contre le code** : la restauration d'une sauvegarde (`backup.rs:457`) et la réinitialisation de démonstration effacent aussi des écritures ⇒ « en usage courant, hors restauration » — et `admin-manual.tex:1802`, même défaut, **réécrit aussi** (l'AC 6 disait de ne pas y toucher) ; ⑤ MEDIUM — le repli Svelte de `fiscal-year-reopen-confirmation-body` (`fiscal-years/+page.svelte:554`) non propagé ⇒ inscrit à l'AC 2 et aux zones touchées. LOW : l'encadré a **cinq** énoncés (le verrou de période reste) ; `:543` classé deux fois ; `admin-manual.tex:1766`, `:1816`, `:1969` et les accords de `user-manual.tex:1809-1812` ajoutés ; préambule périmé réécrit ; AC 5 complétée (numéro invalide ignoré, « — », plage inversée à l'export → #469) ; `user-manual.tex:536` (« Si vous supprimez la dernière écriture ») ajouté à l'AC 4 ; `\label{sec:tracabilite}` prescrit ; bruit du motif `journal` / `protokoll` dit. Ces deux entrées-ci et les deux précédentes, d'abord insérées par erreur dans le tableau de tendance de la passe 7, sont **remontées** en tête du Change Log.
- **2026-09-26** — **réécriture T0** — Après la 25-1c-b1 (revue close), la fiche est **réécrite contre le texte de `main`** : AC 2 réduite à **une clé, trois cellules** (la 25-5-a a aligné l'autre) ; AC 3 : **inventaire refait** (16 sites à changer, dont les « piste de contrôle » ajoutés depuis, et la liste des sites déjà conformes ou d'autre sens) ; AC 4 : l'encadré a **quatre** énoncés, trois périmés, et la seule disparition d'une écriture est la **dévalidation** ; AC 5 : l'écran décrit tel que la b1 le livre (dont le refus de la plage inversée et le vidage du numéro au changement de type, ajoutés par sa revue) ; AC 6 : ce qui reste à corriger au manuel d'administration (« reste à venir », « 5 rôles », `:1803`) ; AC 7 : la ligne v0.12.1 du README et une entrée CHANGELOG *Added* — ⚠️ l'historique publié ne se réécrit pas. Revalidation à passer.
- **2026-09-26** — **revalidation R5 « dérive »** — Reprise sur `main` à `0e4c2682`. **Une lentille Sonnet**, prompt `25-1c-b-validate-prompt-r5-derive.md`, axes déclarés (non exercés : rejeu complet de la procédure Python de l'AC 3/8, manuels DE/IT/EN, PDF de la brochure, CHANGELOG). Constats **vérifiés par l'orchestrateur** (`grep -nF`) avant d'être retenus. ⛔ **CRITICAL** — le mécanisme des AC 4 et 6 n'existe plus : une facture validée ne se supprime plus (`invoices.rs:1293`, `DbError::InvoiceMustBeUnvalidatedFirst`), elle se **dévalide** (`invoice.unvalidated`) ; le code cité (`invoices.rs:1339-1350`) a disparu ; #381 est **fermée**. **HIGH** — (1) `admin-manual.tex` « sans exception » : déjà réécrit ailleurs, la formule n'existe plus à ce site ; (2) item « Champs » (`:1790`) déjà corrigé par la 25-1c-a — reste « L'écran, lui, reste à venir (issue #378) » ; (3) encadré du manuel utilisateur (`:511-528`) déjà réécrit par la 25-1c-a — l'énoncé du verrou de période a disparu ; (4) AC 2 : `export-global-content-excludes` réécrite par la 25-5-a dans les quatre locales, **déjà conforme** — reste une seule clé à aligner (`fiscal-year-reopen-confirmation-body`, fr/it/en). **MEDIUM** — 97 actions et 138 clés `audit-log-*` (non 92 / 133) ; inventaire lexical plus large (nouveaux « piste de contrôle » `admin-manual.tex:1589`, `:1760`, `:1817`, `:1946`, et `user-manual.tex:1111` reformulée) ; toutes les lignes des manuels décalées ; la phrase du README citée par l'AC 7 n'existe plus (la ligne v0.12.1 porte « Restent ouverts : l'écran de consultation ([#378]) »). **LOW** — les entrées de glossaire portent un avertissement ajouté. ⇒ **Décision de l'orchestrateur** : pas de réécriture partielle maintenant — la fiche se réécrit en T0, contre le texte réel, **après** la 25-1c-b1, puis se revalide.
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

# Makefile — build des manuels Kesh LaTeX
# Compile via xelatex (fontspec requis)
#
# Usage :
#   make             — build tous les PDF FR (canoniques)
#   make fr          — idem (alias)
#   make admin       — build uniquement le manuel administrateur FR
#   make user        — build uniquement le manuel utilisateur FR
#   make brochure    — build uniquement la brochure marketing FR
#   make all-langs   — build tous les manuels dans toutes les langues disponibles (FR + traductions)
#   make clean       — supprime les fichiers temporaires LaTeX
#   make distclean   — supprime aussi les PDF générés
#
# Prérequis :
#   - xelatex (TeX Live ≥ 2021 recommandé)
#   - Polices : TeX Gyre Heros (inclus par défaut). Optionnel : Inter + JetBrains Mono
#     (sudo apt install fonts-inter fonts-jetbrains-mono sur Debian/Ubuntu)

LATEX        := xelatex
LATEX_FLAGS  := -interaction=nonstopmode -halt-on-error -file-line-error
# Passes jusqu'à STABILISATION (Story 27-1, revue de code P2, C-27-1-27) : deux
# passes fixes laissaient des tables des matières périmées — LaTeX ne signale
# PAS une `.toc` qui a changé, seulement certaines références (« Rerun »). On
# relance donc tant que `.aux`, `.toc` ou `.out` diffèrent de la passe
# précédente, ou que le journal réclame une nouvelle passe (« Rerun to get… »,
# « Rerun LaTeX », « Please rerun » — non le mot seul, que porte la bannière du
# paquet `rerunfilecheck`) ; au-delà de
# MAX_PASSES, échec bruyant plutôt qu'un PDF faux.
MIN_PASSES   := 2
MAX_PASSES   := 6

LANGS_AVAILABLE := fr
LANGS_FUTURE    := de it en

DOCS := admin-manual user-manual marketing-brochure

# Cibles par défaut : tous les PDF FR
.PHONY: all fr admin user brochure all-langs clean distclean help

all: fr

help:
	@echo "Cibles Makefile manuels Kesh :"
	@echo "  make             — build tous les PDF FR"
	@echo "  make fr          — alias"
	@echo "  make admin       — manuel administrateur FR"
	@echo "  make user        — manuel utilisateur FR"
	@echo "  make brochure    — brochure marketing FR"
	@echo "  make all-langs   — toutes les langues disponibles"
	@echo "  make clean       — supprime fichiers temporaires"
	@echo "  make distclean   — supprime aussi les PDF"

fr: fr/admin-manual.pdf fr/user-manual.pdf fr/marketing-brochure.pdf

admin: fr/admin-manual.pdf

user: fr/user-manual.pdf

brochure: fr/marketing-brochure.pdf

all-langs:
	@for lang in $(LANGS_AVAILABLE); do \
		echo "═══ Build langue $$lang ═══"; \
		for doc in $(DOCS); do \
			if [ -f "$$lang/$$doc.tex" ]; then \
				$(MAKE) "$$lang/$$doc.pdf"; \
			else \
				echo "Skip $$lang/$$doc.tex (non créé, voir $$lang/README.md)"; \
			fi; \
		done; \
	done
	@echo ""
	@echo "Langues futures (stubs) :"
	@for lang in $(LANGS_FUTURE); do \
		echo "  - $$lang/ : voir $$lang/README.md pour ajouter la traduction"; \
	done

# Règle générique : *.tex → *.pdf via xelatex, passes jusqu'à stabilisation
%.pdf: %.tex shared/kesh-style.sty shared/kesh-logo.svg
	@echo "═══ Building $@ (passes jusqu'à stabilisation, $(MAX_PASSES) au plus) ═══"
	@cd $(dir $<) && b=$(basename $(notdir $<)) && n=0 && prev= && \
	while :; do \
		n=$$((n+1)); \
		$(LATEX) $(LATEX_FLAGS) -output-directory=. $(notdir $<) > /dev/null \
			|| { tail -40 $$b.log; echo "✗ $@ : xelatex a échoué (passe $$n)" >&2; exit 1; }; \
		cur=$$(cat $$b.aux $$b.toc $$b.out 2>/dev/null | md5sum); \
		if [ $$n -ge $(MIN_PASSES) ] && [ "$$cur" = "$$prev" ] \
			&& ! grep -qE 'Rerun to get|Rerun LaTeX|Please rerun|Label\(s\) may have changed' $$b.log; then break; fi; \
		if [ $$n -ge $(MAX_PASSES) ]; then \
			echo "✗ $@ : instable après $$n passes" >&2; exit 1; fi; \
		prev=$$cur; \
	done; \
	echo "✓ $@ généré ($$n passes, stable)"

# Cleanup
clean:
	@find . -type f \( \
		-name '*.aux' -o \
		-name '*.log' -o \
		-name '*.out' -o \
		-name '*.toc' -o \
		-name '*.lof' -o \
		-name '*.lot' -o \
		-name '*.fls' -o \
		-name '*.fdb_latexmk' -o \
		-name '*.synctex.gz' -o \
		-name '*.bbl' -o \
		-name '*.blg' -o \
		-name '*.idx' -o \
		-name '*.ind' -o \
		-name '*.ilg' \
	\) -delete
	@echo "✓ Fichiers temporaires LaTeX supprimés"

distclean: clean
	@find . -name '*.pdf' -type f -delete
	@echo "✓ PDF supprimés"

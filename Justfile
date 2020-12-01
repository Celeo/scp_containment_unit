default:

format:
  poetry run black -l 120 scp_containment_unit

run:
  poetry run python scp_containment_unit/__init__.py

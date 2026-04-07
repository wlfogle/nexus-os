# Awesome Stack Ansible Automation

This directory automates your entire stack, mirroring all documented services and infrastructure.

## Structure

- `inventories/production.yml`: Real host/container inventory
- `playbooks/`: One playbook per stack layer, plus all-in-one
- `roles/`: One role per major service/container

## Quickstart

1. Edit `inventories/production.yml` for your IPs and hostnames.
2. Populate each role’s `tasks/main.yml` using your real configs/scripts.
3. Run the full stack:
   ```bash
   ansible-playbook playbooks/all_services.yml -i inventories/production.yml
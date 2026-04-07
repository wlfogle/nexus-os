# Awesome Stack Ansible

This directory contains Ansible automation for:
- Golden LXC template builds
- Agent orchestration
- Service management (Ollama, Home Assistant, etc)
- Git → LLM pipeline automation

## Quickstart

1. Edit `inventories/production.yml` for your hosts.
2. Fill in roles as needed (see `roles/*/tasks/main.yml` for TODOs).
3. Run playbooks as needed, e.g.:
   ```bash
   ansible-playbook playbooks/golden_image.yml -i inventories/production.yml
   ```

## AWX Integration

You can import and chain these playbooks into an AWX workflow for scheduled or event-driven automation.

**See `../docs/AWX_workflow_example.md` for details.**
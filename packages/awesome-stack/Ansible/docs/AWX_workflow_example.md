# AWX Workflow Example

## Sample Workflow

1. Build golden LXC template (`golden_image.yml`)
2. Deploy agent system (`agent_orchestration.yml`)
3. Deploy core services (`service_management.yml`)
4. Manage Ollama/LLM endpoint (`ollama_management.yml`)
5. Manage Home Assistant VM (`home_assistant_management.yml`)
6. Run Git→LLM pipeline (`git_llm_pipeline.yml`)

## Example (YAML)

```yaml
- name: Ultimate Media Stack Build
  workflow:
    - job_template: Build Golden LXC
    - job_template: Deploy Agent System
    - job_template: Deploy Core Services
    - job_template: Manage Ollama Endpoint
    - job_template: Manage Home Assistant
    - job_template: Git→LLM Pipeline
```

**Import each playbook above as a Job Template in AWX, then link them in this order.**

---

## Customize

- Fill in the TODOs in each role with your environment specifics.
- Add secrets and credentials to AWX’s vault or environment variables for safety.
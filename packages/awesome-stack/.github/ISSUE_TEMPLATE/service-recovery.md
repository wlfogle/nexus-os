---
name: 🚨 Service Recovery
about: Track recovery of stopped or failed services
title: '[SERVICE-RECOVERY] Fix {{SERVICE_NAME}} in {{CONTAINER_ID}}'
labels: ['bug', 'critical', 'infrastructure', 'service-recovery']
assignees: ''
---

## 🚨 Service Recovery Request

### Service Details
- **Container ID**: CT-XXX
- **Service Name**: Service Name
- **Current Status**: ❌ Stopped / ⚠️ Partial / 🔄 Starting
- **Priority**: Critical / High / Medium / Low
- **Discovery Date**: YYYY-MM-DD

### Problem Description
Brief description of what's wrong with the service.

### Impact Assessment
- [ ] **Critical**: Core functionality broken
- [ ] **High**: Major features unavailable  
- [ ] **Medium**: Convenience features missing
- [ ] **Low**: Minor enhancements unavailable

### User Impact
Describe how this affects end users:
- What functionality is unavailable?
- What workarounds exist?
- How many users affected?

### Recovery Steps
- [ ] Check container status: `pct status XXX`
- [ ] Check service logs: `pct exec XXX -- journalctl -u service-name`
- [ ] Attempt service restart: `pct exec XXX -- systemctl restart service-name`
- [ ] Verify service health: `pct exec XXX -- systemctl status service-name`
- [ ] Test functionality: Access web UI / API
- [ ] Update monitoring: Confirm metrics/alerts working

### Dependencies
List any services that depend on this one:
- Service A (CT-XXX)
- Service B (CT-XXX)

### Related Issues
Link any related issues or PRs:
- #XXX - Related issue
- #XXX - Dependency issue

### Acceptance Criteria
- [ ] Service is actively running
- [ ] Web UI/API is accessible
- [ ] All core functionality working
- [ ] Dependencies are satisfied
- [ ] Monitoring/alerts configured
- [ ] Documentation updated

### Recovery Verification
```bash
# Commands to verify service is fully recovered
pct status XXX
pct exec XXX -- systemctl status service-name
curl -f http://service-url/health || echo "Health check failed"
```

### Notes
Additional context, logs, or observations.

"""
RustAgent adapter for LongHorizon-Harness.

This module provides an AgentAdapter implementation that calls the RustAgent
binary in headless CLI mode.

Usage in .lh-harness/config.toml:
    [run]
    agent = "rustagent"
    model = "gpt-4o"

Or register as a custom adapter in your harness setup.

Command template used:
    RustAgent --profile headless --prompt-file {prompt_path} --timeout {timeout} --auto-approve
"""

from __future__ import annotations

import shlex
import time
import posixpath
from pathlib import PurePath

from lh_harness.environment.base import Environment
from lh_harness.types import EpisodeBudget, EpisodeResult, DEFAULT_TMP_DIR


class RustAgentAdapter:
    """AgentAdapter implementation for RustAgent binary.

    Calls RustAgent in headless mode via CLI, passing the prompt through
    a file and capturing stdout as the agent's response.
    """

    def __init__(
        self,
        *,
        binary: str = "RustAgent",
        profile: str = "headless",
        model: str | None = None,
        workspace_path: str = ".",
        prompt_dir: str = f"{DEFAULT_TMP_DIR}/prompts",
        auto_approve: bool = True,
        read_only: bool = False,
    ) -> None:
        self.binary = binary
        self.profile = profile
        self.model = model
        self.workspace_path = workspace_path.rstrip("/")
        self.prompt_dir = prompt_dir.rstrip("/") or "."
        self.auto_approve = auto_approve
        self.read_only = read_only

    async def run_episode(
        self,
        prompt: str,
        env: Environment,
        budget: EpisodeBudget,
        live_trajectory_path: str | None = None,
    ) -> EpisodeResult:
        start = time.monotonic()

        # Write prompt to a temporary file
        import uuid
        prompt_filename = f"rustagent_{uuid.uuid4().hex[:12]}.md"
        prompt_path = posixpath.join(self.prompt_dir, prompt_filename)

        # Use env.exec to write the prompt file
        await env.exec(
            f"mkdir -p {shlex.quote(self.prompt_dir)} && cat > {shlex.quote(prompt_path)}",
            timeout=10,
        )
        # Write prompt content via heredoc
        escaped_prompt = prompt.replace("'", "'\\''")
        await env.exec(
            f"printf '%s' '{escaped_prompt}' > {shlex.quote(prompt_path)}",
            timeout=10,
        )

        # Build the RustAgent command
        cmd_parts = [
            self.binary,
            "--profile", self.profile,
            "--prompt-file", prompt_path,
            "--timeout", str(budget.max_duration_seconds),
        ]

        if self.auto_approve and not self.read_only:
            cmd_parts.append("--auto-approve")
        if self.read_only:
            cmd_parts.append("--read-only")
        if self.model:
            cmd_parts.extend(["--model", self.model])
        if live_trajectory_path:
            cmd_parts.extend(["--trajectory", live_trajectory_path])

        command = f"cd {shlex.quote(self.workspace_path)} && {shlex.join(cmd_parts)}"

        # Execute
        result = await env.exec(
            command,
            timeout=budget.max_duration_seconds,
            tee_path=live_trajectory_path,
        )

        duration_ms = int((time.monotonic() - start) * 1000)

        # Determine status
        if result.termination_reason == "timeout":
            status = "timeout"
        elif result.exit_code == 0:
            status = "done"
        else:
            status = "error"

        # Build error message
        error = None
        if result.termination_reason == "timeout":
            error = f"Episode timed out after {budget.max_duration_seconds}s."
        elif result.exit_code != 0:
            error = result.stderr[-2000:] if result.stderr else f"Exit code {result.exit_code}"

        return EpisodeResult(
            status=status,
            actions_log=result.stdout,
            error=error,
            duration_ms=duration_ms,
            metadata={
                "command": command,
                "workspace": self.workspace_path,
                "prompt_path": prompt_path,
                "exit_code": result.exit_code,
                "termination_reason": result.termination_reason,
                "agent": "rustagent",
                "profile": self.profile,
            },
        )


# ── CommandAgentAdapter-compatible template ──
# For use with LongHorizon-Harness's built-in CommandAgentAdapter:
#
# RUSTAGENT_COMMAND_TEMPLATE = (
#     "RustAgent --profile headless --prompt-file {prompt_path} "
#     "--timeout {timeout} --auto-approve"
# )
#
# adapter = CommandAgentAdapter(
#     command_template=RUSTAGENT_COMMAND_TEMPLATE,
#     workspace_path="/path/to/project",
# )

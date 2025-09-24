#!/usr/bin/env python3
"""
Claude Manager TUI - Type-safe visual session management for Claude
Enhanced with Python 3.13+ type safety features and comprehensive error handling
"""

import sys
import json
import asyncio
import subprocess
import re
from pathlib import Path
from datetime import datetime
from dataclasses import dataclass, field
from typing import (
    List, Dict, Tuple, Optional, Union, Any, Literal, 
    Generic, TypeVar, TypeIs, NewType, ReadOnly, Protocol,
    Awaitable, AsyncIterator, Final
)
from typing_extensions import Self
from enum import Enum, auto
from contextlib import asynccontextmanager

from rich.console import Console
from rich.table import Table
from rich.panel import Panel
from rich.layout import Layout
from rich.live import Live
from rich.text import Text
from rich.syntax import Syntax
from prompt_toolkit import prompt
from prompt_toolkit.shortcuts import radiolist_dialog, yes_no_dialog
from prompt_toolkit.completion import WordCompleter
from prompt_toolkit.formatted_text import HTML

# Type definitions for domain-specific strings
SessionName = NewType('SessionName', str)
SessionPath = NewType('SessionPath', str)
WorkingDirectory = NewType('WorkingDirectory', str)
CommandOutput = NewType('CommandOutput', str)
SearchQuery = NewType('SearchQuery', str)

# Generic type variables
T = TypeVar('T')
E = TypeVar('E', bound=Exception)

# Console instance
console: Final[Console] = Console()

class ClaudeManagerError(Exception):
    """Base exception for Claude Manager operations"""
    pass

class SessionLoadError(ClaudeManagerError):
    """Error loading Claude sessions"""
    pass

class CommandExecutionError(ClaudeManagerError):
    """Error executing cm command"""
    pass

class JsonParsingError(ClaudeManagerError):
    """Error parsing JSON session data"""
    pass

class SessionMigrationError(ClaudeManagerError):
    """Error during session migration"""
    pass

@dataclass(frozen=True, slots=True)
class Result(Generic[T, E]):
    """Type-safe result type for operations that can fail"""
    value: Optional[T] = None
    error: Optional[E] = None
    
    @property
    def is_success(self) -> bool:
        return self.error is None and self.value is not None
    
    @property
    def is_failure(self) -> bool:
        return self.error is not None
    
    def unwrap(self) -> T:
        """Get value or raise the error"""
        if self.is_failure:
            raise self.error  # type: ignore
        return self.value  # type: ignore
    
    def unwrap_or(self, default: T) -> T:
        """Get value or return default"""
        return self.value if self.is_success else default

@dataclass(frozen=True, slots=True)
class SessionMetadata:
    """Metadata extracted from session JSON"""
    cwd: Optional[WorkingDirectory]
    last_modified: Optional[datetime]
    total_messages: int = 0
    
    @classmethod
    def from_json_data(cls, data: Dict[str, Any]) -> Self:
        """Create metadata from parsed JSON data"""
        cwd = WorkingDirectory(data.get('cwd', '')) if data.get('cwd') else None
        
        # Parse timestamp if available
        last_modified = None
        if 'timestamp' in data:
            try:
                last_modified = datetime.fromisoformat(data['timestamp'])
            except ValueError:
                pass
                
        return cls(
            cwd=cwd,
            last_modified=last_modified,
            total_messages=data.get('message_count', 0)
        )

@dataclass(frozen=True, slots=True)
class ClaudeSession:
    """Represents a Claude session/project with comprehensive type safety"""
    name: SessionName
    path: SessionPath
    session_count: int
    metadata: SessionMetadata = field(default_factory=lambda: SessionMetadata(None, None))
    
    @property
    def display_name(self) -> str:
        """Clean up the encoded directory name for display"""
        return self.name.replace('-Users-tryk-', '~/').replace('-', '/')
    
    @property
    def current_cwd(self) -> str:
        """Get current working directory or empty string"""
        return self.metadata.cwd or ""
    
    def is_aura_session(self) -> bool:
        """Check if this is an aura session"""
        return 'auras' in self.name.lower()

class SessionValidator:
    """Type-safe session validation utilities"""
    
    @staticmethod
    def is_valid_session_name(value: str) -> TypeIs[SessionName]:
        """Type guard for valid session names"""
        return bool(value and isinstance(value, str) and len(value.strip()) > 0)
    
    @staticmethod
    def is_valid_session_path(value: str) -> TypeIs[SessionPath]:
        """Type guard for valid session paths"""
        if not value or not isinstance(value, str):
            return False
        expanded = os.path.expanduser(value)
        return Path(expanded).exists()
    
    @staticmethod
    def is_valid_working_directory(value: str) -> TypeIs[WorkingDirectory]:
        """Type guard for valid working directories"""
        if value == "":
            return True
        if not isinstance(value, str):
            return False
        expanded = os.path.expanduser(value)
        return Path(expanded).is_dir()

@dataclass(frozen=True, slots=True)
class ClaudeManagerConfig:
    """Immutable configuration for Claude Manager"""
    claude_dir: ReadOnly[Path]
    cm_command: ReadOnly[str] = "cm"
    max_concurrent_sessions: ReadOnly[int] = 10
    session_load_timeout: ReadOnly[float] = 30.0
    
    @classmethod
    def from_environment(cls) -> Self:
        """Create configuration from environment variables"""
        import os
        claude_dir = Path(os.getenv('CLAUDE_DIR', Path.home() / '.claude' / 'projects'))
        cm_command = os.getenv('CM_COMMAND') or os.getenv('CLAUDE_MANAGER_BIN', 'cm')
        return cls(
            claude_dir=claude_dir,
            cm_command=cm_command,
            max_concurrent_sessions=int(os.getenv('CLAUDE_MAX_SESSIONS', '10')),
            session_load_timeout=float(os.getenv('CLAUDE_SESSION_TIMEOUT', '30.0'))
        )
    
    @classmethod
    def default(cls) -> Self:
        """Create default configuration"""
        return cls(claude_dir=Path.home() / ".claude" / "projects")

class SearchResult:
    """Represents a search result with scoring"""
    
    def __init__(self, session: ClaudeSession, score: int, matches: List[str]) -> None:
        self.session = session
        self.score = score
        self.matches = matches
    
    def __lt__(self, other: 'SearchResult') -> bool:
        return self.score < other.score
    
    def __eq__(self, other: object) -> bool:
        return isinstance(other, SearchResult) and self.score == other.score

class ClaudeManager:
    """Type-safe Claude Manager interface with async operations"""
    
    def __init__(self, config: Optional[ClaudeManagerConfig] = None) -> None:
        self.config = config or ClaudeManagerConfig.default()
        self.sessions: List[ClaudeSession] = []
        self._session_cache: Dict[SessionName, ClaudeSession] = {}
    
    async def run_cm_command_async(self, *args: str) -> Result[CommandOutput, CommandExecutionError]:
        """Run cm command asynchronously and return typed result"""
        try:
            process = await asyncio.create_subprocess_exec(
                self.config.cm_command, *args,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE
            )
            
            stdout_bytes, stderr_bytes = await asyncio.wait_for(
                process.communicate(),
                timeout=self.config.session_load_timeout
            )
            
            # Decode bytes to text
            stdout = stdout_bytes.decode('utf-8', errors='replace') if stdout_bytes else ""
            stderr = stderr_bytes.decode('utf-8', errors='replace') if stderr_bytes else ""
            
            if process.returncode != 0:
                error_msg = f"Command failed with code {process.returncode}: {stderr}"
                return Result(error=CommandExecutionError(error_msg))
            
            return Result(value=CommandOutput(stdout))
            
        except asyncio.TimeoutError:
            return Result(error=CommandExecutionError("Command timed out"))
        except Exception as e:
            return Result(error=CommandExecutionError(f"Command execution failed: {e}"))
    
    def run_cm_command(self, *args: str) -> Result[CommandOutput, CommandExecutionError]:
        """Synchronous wrapper for cm command execution"""
        try:
            result = subprocess.run(
                [self.config.cm_command] + list(args),
                capture_output=True,
                text=True,
                timeout=self.config.session_load_timeout,
                check=False
            )
            
            if result.returncode != 0:
                error_msg = f"Command failed with code {result.returncode}: {result.stderr}"
                return Result(error=CommandExecutionError(error_msg))
            
            return Result(value=CommandOutput(result.stdout))
            
        except subprocess.TimeoutExpired:
            return Result(error=CommandExecutionError("Command timed out"))
        except Exception as e:
            return Result(error=CommandExecutionError(f"Command execution failed: {e}"))
    
    async def load_session_metadata(self, session_dir: Path) -> Result[SessionMetadata, JsonParsingError]:
        """Load metadata from session JSONL files"""
        try:
            session_files = list(session_dir.glob("*.jsonl"))
            if not session_files:
                return Result(value=SessionMetadata(None, None))
            
            # Read the first session file to get metadata
            session_file = session_files[0]
            
            async with asyncio.TaskGroup() as tg:
                # Create task to read file asynchronously
                task = tg.create_task(self._read_session_file(session_file))
            
            metadata = await task
            return Result(value=metadata)
            
        except Exception as e:
            return Result(error=JsonParsingError(f"Failed to load session metadata: {e}"))
    
    async def _read_session_file(self, file_path: Path) -> SessionMetadata:
        """Read session file and extract metadata"""
        # In a real async implementation, we'd use aiofiles
        # For now, we'll use asyncio.to_thread for file I/O
        def _sync_read() -> SessionMetadata:
            try:
                with open(file_path, 'r', encoding='utf-8') as f:
                    for line in f:
                        if '"cwd":' in line:
                            try:
                                data = json.loads(line)
                                return SessionMetadata.from_json_data(data)
                            except json.JSONDecodeError:
                                continue
                return SessionMetadata(None, None)
            except Exception:
                return SessionMetadata(None, None)
        
        return await asyncio.to_thread(_sync_read)
    
    async def load_sessions_async(self) -> Result[List[ClaudeSession], SessionLoadError]:
        """Load all Claude sessions asynchronously"""
        try:
            # Get session list from cm command
            result = await self.run_cm_command_async('list')
            if result.is_failure:
                return Result(error=SessionLoadError(f"Failed to list sessions: {result.error}"))
            
            output = result.unwrap()
            sessions: List[ClaudeSession] = []
            
            # Parse session information
            for line in output.split('\n'):
                if 'sessions)' in line:
                    session_result = await self._parse_session_line(line)
                    if session_result.is_success:
                        sessions.append(session_result.unwrap())
            
            # Load metadata for all sessions concurrently
            if sessions:
                sessions = await self._load_sessions_metadata(sessions)
            
            self.sessions = sessions
            self._update_session_cache()
            
            return Result(value=sessions)
            
        except Exception as e:
            return Result(error=SessionLoadError(f"Failed to load sessions: {e}"))
    
    async def _parse_session_line(self, line: str) -> Result[ClaudeSession, SessionLoadError]:
        """Parse a single session line from cm list output"""
        try:
            # Parse: "  -Users-tryk-nabia-chats-auras-igris (       7 sessions)"
            match = re.match(r'\s*([^\s]+)\s*\(\s*(\d+)\s+sessions?\)', line)
            if not match:
                return Result(error=SessionLoadError(f"Invalid session line format: {line}"))
            
            name_str = match.group(1)
            count_str = match.group(2)
            
            if not SessionValidator.is_valid_session_name(name_str):
                return Result(error=SessionLoadError(f"Invalid session name: {name_str}"))
            
            try:
                count = int(count_str)
            except ValueError:
                return Result(error=SessionLoadError(f"Invalid session count: {count_str}"))
            
            name = SessionName(name_str)
            path = SessionPath(str(self.config.claude_dir / name))
            
            session = ClaudeSession(
                name=name,
                path=path,
                session_count=count
            )
            
            return Result(value=session)
            
        except Exception as e:
            return Result(error=SessionLoadError(f"Failed to parse session line: {e}"))
    
    async def _load_sessions_metadata(self, sessions: List[ClaudeSession]) -> List[ClaudeSession]:
        """Load metadata for all sessions concurrently"""
        async def load_metadata(session: ClaudeSession) -> ClaudeSession:
            session_dir = Path(session.path)
            metadata_result = await self.load_session_metadata(session_dir)
            
            if metadata_result.is_success:
                metadata = metadata_result.unwrap()
                return ClaudeSession(
                    name=session.name,
                    path=session.path,
                    session_count=session.session_count,
                    metadata=metadata
                )
            else:
                # Keep original session if metadata loading fails
                return session
        
        # Load metadata concurrently but limit concurrency
        semaphore = asyncio.Semaphore(self.config.max_concurrent_sessions)
        
        async def load_with_semaphore(session: ClaudeSession) -> ClaudeSession:
            async with semaphore:
                return await load_metadata(session)
        
        tasks = [load_with_semaphore(session) for session in sessions]
        return await asyncio.gather(*tasks, return_exceptions=False)
    
    def load_sessions(self) -> Result[List[ClaudeSession], SessionLoadError]:
        """Synchronous wrapper for session loading"""
        try:
            return asyncio.run(self.load_sessions_async())
        except Exception as e:
            return Result(error=SessionLoadError(f"Failed to load sessions: {e}"))
    
    def _update_session_cache(self) -> None:
        """Update the session cache for faster lookups"""
        self._session_cache = {session.name: session for session in self.sessions}
    
    def get_session_by_name(self, name: SessionName) -> Optional[ClaudeSession]:
        """Get session by name from cache"""
        return self._session_cache.get(name)
    
    def display_sessions_table(self) -> None:
        """Display sessions in a categorized table"""
        table = Table(title="Claude Sessions", show_header=True, header_style="bold magenta")
        table.add_column("Name", style="cyan", no_wrap=False)
        table.add_column("Sessions", justify="right", style="green")
        table.add_column("Current Path", style="yellow")
        table.add_column("Last Modified", style="dim")
        
        # Categorize sessions
        aura_sessions = [s for s in self.sessions if s.is_aura_session()]
        other_sessions = [s for s in self.sessions if not s.is_aura_session()]
        
        def add_session_rows(sessions: List[ClaudeSession], category_title: str) -> None:
            if not sessions:
                return
                
            table.add_row(f"[bold]{category_title}[/bold]", "", "", "")
            for session in sorted(sessions, key=lambda x: x.name):
                last_modified = ""
                if session.metadata.last_modified:
                    last_modified = session.metadata.last_modified.strftime("%Y-%m-%d %H:%M")
                
                table.add_row(
                    f"  {session.display_name}",
                    str(session.session_count),
                    session.current_cwd or "N/A",
                    last_modified
                )
        
        add_session_rows(aura_sessions, "Aura Sessions")
        if aura_sessions and other_sessions:
            table.add_row("", "", "", "")  # Separator
        add_session_rows(other_sessions, "Other Sessions")
        
        console.print(table)
    
    async def migrate_session_async(self, session: ClaudeSession, new_path: WorkingDirectory) -> Result[None, SessionMigrationError]:
        """Migrate session to new path asynchronously"""
        try:
            if not SessionValidator.is_valid_working_directory(new_path):
                return Result(error=SessionMigrationError(f"Invalid new path: {new_path}"))
            
            result = await self.run_cm_command_async(
                'migrate',
                session.current_cwd,
                new_path,
                session.path
            )
            
            if result.is_failure:
                return Result(error=SessionMigrationError(f"Migration command failed: {result.error}"))
            
            # Reload sessions after migration
            await self.load_sessions_async()
            
            return Result(value=None)
            
        except Exception as e:
            return Result(error=SessionMigrationError(f"Migration failed: {e}"))
    
    async def migrate_session(self, session: ClaudeSession) -> None:
        """Interactive migration for a session"""
        console.print(f"\n[cyan]Migrating session: {session.display_name}[/cyan]")
        console.print(f"Current path: [yellow]{session.current_cwd}[/yellow]")
        
        # Get new path from user
        new_path_str = prompt(
            HTML('<ansiyellow>Enter new path (or press Enter to cancel): </ansiyellow>'),
            default=session.current_cwd
        )
        
        if new_path_str and new_path_str != session.current_cwd:
            # Validate new path
            if not SessionValidator.is_valid_working_directory(new_path_str):
                console.print(f"[red]Error: Invalid path '{new_path_str}'[/red]")
                return
            
            new_path = WorkingDirectory(new_path_str)
            
            # Confirm migration
            if yes_no_dialog(
                title='Confirm Migration',
                text=f'Migrate from:\n{session.current_cwd}\n\nTo:\n{new_path}'
            ).run():
                
                console.print(f"[green]Migrating session...[/green]")
                
                try:
                    result = await self.migrate_session_async(session, new_path)
                    
                    if result.is_success:
                        console.print(f"[green]✓ Migration complete![/green]")
                    else:
                        console.print(f"[red]Migration failed: {result.error}[/red]")
                        
                except Exception as e:
                    console.print(f"[red]Migration error: {e}[/red]")
    
    def search_sessions(self, query: SearchQuery) -> List[SearchResult]:
        """Search through sessions with enhanced scoring"""
        if not query.strip():
            return []
        
        keywords = query.lower().split()
        results: List[SearchResult] = []
        
        for session in self.sessions:
            score = 0
            matches: List[str] = []
            
            name_lower = session.name.lower()
            path_lower = session.current_cwd.lower() if session.current_cwd else ""
            display_lower = session.display_name.lower()
            
            for keyword in keywords:
                if keyword in name_lower:
                    score += 3
                    matches.append(f"name: {keyword}")
                if keyword in display_lower:
                    score += 2
                    matches.append(f"display: {keyword}")
                if keyword in path_lower:
                    score += 1
                    matches.append(f"path: {keyword}")
            
            if score > 0:
                results.append(SearchResult(session, score, matches))
        
        # Sort by score (descending)
        results.sort(reverse=True)
        return results
    
    def display_search_results(self, query: SearchQuery, results: List[SearchResult]) -> None:
        """Display search results in a formatted way"""
        if not results:
            console.print(f"[yellow]No matches found for '{query}'[/yellow]")
            return
        
        console.print(f"\n[green]Found {len(results)} matches for '{query}':[/green]\n")
        
        for i, result in enumerate(results[:10], 1):
            session = result.session
            console.print(f"{i:2d}. [cyan]{session.display_name}[/cyan] (score: {result.score})")
            console.print(f"    Path: [yellow]{session.current_cwd or 'N/A'}[/yellow]")
            console.print(f"    Sessions: [green]{session.session_count}[/green]")
            if result.matches:
                console.print(f"    Matches: [dim]{', '.join(result.matches)}[/dim]")
            console.print()
    
    async def interactive_menu_async(self) -> None:
        """Main interactive menu with async operations"""
        while True:
            console.clear()
            console.print(Panel.fit(
                "[bold cyan]Claude Manager TUI (Type-Safe)[/bold cyan]\n"
                "Visual session management for Claude with comprehensive type safety",
                border_style="cyan"
            ))
            
            # Load sessions if not already loaded
            if not self.sessions:
                console.print("[yellow]Loading sessions...[/yellow]")
                result = await self.load_sessions_async()
                if result.is_failure:
                    console.print(f"[red]Failed to load sessions: {result.error}[/red]")
                    break
            
            self.display_sessions_table()
            
            # Menu options
            choices = [
                ('migrate', 'Migrate a session'),
                ('search', 'Search sessions'),
                ('refresh', 'Refresh session list'),
                ('quit', 'Exit')
            ]
            
            result = radiolist_dialog(
                title="Claude Manager",
                text="What would you like to do?",
                values=choices
            ).run()
            
            if result == 'quit' or result is None:
                break
            elif result == 'migrate':
                await self._handle_migration_menu()
            elif result == 'search':
                await self._handle_search_menu()
            elif result == 'refresh':
                console.print("[green]Refreshing sessions...[/green]")
                await self.load_sessions_async()
    
    async def _handle_migration_menu(self) -> None:
        """Handle the migration menu interaction"""
        # Select session to migrate
        session_choices = [
            (s, s.display_name) for s in self.sessions
        ]
        selected = radiolist_dialog(
            title="Select Session",
            text="Choose a session to migrate:",
            values=session_choices
        ).run()
        
        if selected:
            await self.migrate_session(selected)
    
    async def _handle_search_menu(self) -> None:
        """Handle the search menu interaction"""
        query_str = prompt(HTML('<ansicyan>Search query: </ansicyan>'))
        if query_str:
            query = SearchQuery(query_str)
            results = self.search_sessions(query)
            self.display_search_results(query, results)
            prompt(HTML('<ansigreen>Press Enter to continue...</ansigreen>'))
    
    def interactive_menu(self) -> None:
        """Synchronous wrapper for interactive menu"""
        try:
            asyncio.run(self.interactive_menu_async())
        except KeyboardInterrupt:
            console.print("\n[yellow]Goodbye![/yellow]")
        except Exception as e:
            console.print(f"\n[red]Error: {e}[/red]")

async def main_async() -> None:
    """Async main entry point"""
    config = ClaudeManagerConfig.from_environment()
    manager = ClaudeManager(config)
    
    if len(sys.argv) > 1:
        # Command line mode
        command = sys.argv[1]
        if command == 'list':
            result = await manager.load_sessions_async()
            if result.is_success:
                manager.display_sessions_table()
            else:
                console.print(f"[red]Error loading sessions: {result.error}[/red]")
                
        elif command == 'search' and len(sys.argv) > 2:
            query = SearchQuery(' '.join(sys.argv[2:]))
            
            # Load sessions first
            result = await manager.load_sessions_async()
            if result.is_failure:
                console.print(f"[red]Error loading sessions: {result.error}[/red]")
                return
            
            search_results = manager.search_sessions(query)
            manager.display_search_results(query, search_results)
        else:
            console.print("[red]Unknown command. Available: list, search <query>[/red]")
    else:
        # Interactive mode
        await manager.interactive_menu_async()

def main() -> None:
    """Main entry point"""
    try:
        asyncio.run(main_async())
    except KeyboardInterrupt:
        console.print("\n[yellow]Goodbye![/yellow]")
    except Exception as e:
        console.print(f"\n[red]Fatal error: {e}[/red]")
        sys.exit(1)

if __name__ == "__main__":
    main()
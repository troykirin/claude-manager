#!/usr/bin/env python3
"""
Claude Manager TUI - Visual session management for Claude
Following the riff-cli pattern for intent-driven search and management
"""

import sys
import json
import subprocess
import re
from pathlib import Path
from datetime import datetime
from typing import List, Dict, Tuple, Optional
from dataclasses import dataclass

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

console = Console()

@dataclass
class ClaudeSession:
    """Represents a Claude session/project"""
    name: str
    path: str
    session_count: int
    current_cwd: str = ""
    last_modified: Optional[datetime] = None
    
    @property
    def display_name(self):
        """Clean up the encoded directory name for display"""
        return self.name.replace('-Users-tryk-', '~/').replace('-', '/')

class ClaudeManager:
    """Main Claude Manager interface"""
    
    def __init__(self):
        self.claude_dir = Path.home() / ".claude" / "projects"
        self.sessions = []
        self.load_sessions()
    
    def run_cm_command(self, *args) -> str:
        """Run cm command and return output"""
        try:
            result = subprocess.run(
                ['cm'] + list(args),
                capture_output=True,
                text=True,
                check=False
            )
            return result.stdout
        except Exception as e:
            console.print(f"[red]Error running cm: {e}[/red]")
            return ""
    
    def load_sessions(self):
        """Load all Claude sessions"""
        output = self.run_cm_command('list')
        self.sessions = []
        
        for line in output.split('\n'):
            if 'sessions)' in line:
                # Parse: "  -Users-tryk-nabia-chats-auras-igris (       7 sessions)"
                match = re.match(r'\s*([^\s]+)\s*\(\s*(\d+)\s+sessions?\)', line)
                if match:
                    name = match.group(1)
                    count = int(match.group(2))
                    
                    # Get the actual path by checking for cwd in sessions
                    session = ClaudeSession(
                        name=name,
                        path=str(self.claude_dir / name),
                        session_count=count
                    )
                    
                    # Try to get the current working directory from the session
                    session_files = list((self.claude_dir / name).glob("*.jsonl"))
                    if session_files:
                        try:
                            with open(session_files[0], 'r') as f:
                                for line_content in f:
                                    if '"cwd":' in line_content:
                                        data = json.loads(line_content)
                                        if 'cwd' in data:
                                            session.current_cwd = data['cwd']
                                            break
                        except:
                            pass
                    
                    self.sessions.append(session)
    
    def display_sessions_table(self):
        """Display sessions in a nice table"""
        table = Table(title="Claude Sessions", show_header=True, header_style="bold magenta")
        table.add_column("Name", style="cyan", no_wrap=False)
        table.add_column("Sessions", justify="right", style="green")
        table.add_column("Current Path", style="yellow")
        
        # Group sessions by category
        aura_sessions = [s for s in self.sessions if 'auras' in s.name]
        other_sessions = [s for s in self.sessions if 'auras' not in s.name]
        
        if aura_sessions:
            table.add_row("[bold]Aura Sessions[/bold]", "", "")
            for session in sorted(aura_sessions, key=lambda x: x.name):
                table.add_row(
                    f"  {session.display_name}",
                    str(session.session_count),
                    session.current_cwd or "N/A"
                )
        
        if other_sessions:
            if aura_sessions:
                table.add_row("", "", "")  # Separator
            table.add_row("[bold]Other Sessions[/bold]", "", "")
            for session in sorted(other_sessions, key=lambda x: x.name):
                table.add_row(
                    f"  {session.display_name}",
                    str(session.session_count),
                    session.current_cwd or "N/A"
                )
        
        console.print(table)
    
    def migrate_session(self, session: ClaudeSession):
        """Interactive migration for a session"""
        console.print(f"\n[cyan]Migrating session: {session.display_name}[/cyan]")
        console.print(f"Current path: [yellow]{session.current_cwd}[/yellow]")
        
        # Get new path from user
        new_path = prompt(
            HTML('<ansiyellow>Enter new path (or press Enter to cancel): </ansiyellow>'),
            default=session.current_cwd
        )
        
        if new_path and new_path != session.current_cwd:
            # Confirm migration
            if yes_no_dialog(
                title='Confirm Migration',
                text=f'Migrate from:\n{session.current_cwd}\n\nTo:\n{new_path}'
            ).run():
                
                console.print(f"[green]Migrating session...[/green]")
                
                # Run the migration
                output = self.run_cm_command(
                    'migrate',
                    session.current_cwd,
                    new_path,
                    str(self.claude_dir / session.name)
                )
                
                console.print(output)
                console.print(f"[green]✓ Migration complete![/green]")
                
                # Reload sessions
                self.load_sessions()
    
    def search_sessions(self, query: str):
        """Search through sessions with intent-driven enhancement"""
        # Simple search for now - can be enhanced with intent patterns
        keywords = query.lower().split()
        matches = []
        
        for session in self.sessions:
            score = 0
            name_lower = session.name.lower()
            path_lower = session.current_cwd.lower() if session.current_cwd else ""
            
            for keyword in keywords:
                if keyword in name_lower:
                    score += 2
                if keyword in path_lower:
                    score += 1
            
            if score > 0:
                matches.append((session, score))
        
        # Sort by score
        matches.sort(key=lambda x: x[1], reverse=True)
        
        if matches:
            console.print(f"\n[green]Found {len(matches)} matches:[/green]")
            for session, score in matches[:10]:
                console.print(f"  • {session.display_name} (score: {score})")
        else:
            console.print(f"[yellow]No matches found for '{query}'[/yellow]")
    
    def interactive_menu(self):
        """Main interactive menu"""
        while True:
            console.clear()
            console.print(Panel.fit(
                "[bold cyan]Claude Manager TUI[/bold cyan]\n"
                "Visual session management for Claude",
                border_style="cyan"
            ))
            
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
                    self.migrate_session(selected)
            elif result == 'search':
                query = prompt(HTML('<ansicyan>Search query: </ansicyan>'))
                if query:
                    self.search_sessions(query)
                    prompt(HTML('<ansigreen>Press Enter to continue...</ansigreen>'))
            elif result == 'refresh':
                console.print("[green]Refreshing sessions...[/green]")
                self.load_sessions()

def main():
    """Main entry point"""
    manager = ClaudeManager()
    
    if len(sys.argv) > 1:
        # Command line mode
        command = sys.argv[1]
        if command == 'list':
            manager.display_sessions_table()
        elif command == 'search' and len(sys.argv) > 2:
            query = ' '.join(sys.argv[2:])
            manager.search_sessions(query)
        else:
            console.print("[red]Unknown command[/red]")
    else:
        # Interactive mode
        try:
            manager.interactive_menu()
        except KeyboardInterrupt:
            console.print("\n[yellow]Goodbye![/yellow]")

if __name__ == "__main__":
    main()
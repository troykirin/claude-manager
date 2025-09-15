#!/usr/bin/env python3
"""
Comprehensive pytest test suite for Claude Manager TUI
Demonstrates type-safe testing patterns and comprehensive coverage
"""

import pytest
import asyncio
import json
import tempfile
from pathlib import Path
from datetime import datetime
from unittest.mock import Mock, patch, AsyncMock, MagicMock
from typing import List, Dict, Any, Optional

from claude_manager_tui_typed import (
    ClaudeManager, ClaudeSession, SessionMetadata, ClaudeManagerConfig,
    SessionName, SessionPath, WorkingDirectory, CommandOutput, SearchQuery,
    Result, ClaudeManagerError, SessionLoadError, CommandExecutionError,
    JsonParsingError, SessionMigrationError, SessionValidator, SearchResult
)

# Test fixtures

@pytest.fixture
def temp_claude_dir() -> Path:
    """Create a temporary Claude directory for testing"""
    with tempfile.TemporaryDirectory() as temp_dir:
        claude_dir = Path(temp_dir) / ".claude" / "projects"
        claude_dir.mkdir(parents=True)
        yield claude_dir

@pytest.fixture
def test_config(temp_claude_dir: Path) -> ClaudeManagerConfig:
    """Create test configuration"""
    return ClaudeManagerConfig(
        claude_dir=temp_claude_dir,
        cm_command="echo",  # Use echo for testing
        max_concurrent_sessions=5,
        session_load_timeout=1.0
    )

@pytest.fixture
def sample_session() -> ClaudeSession:
    """Create a sample session for testing"""
    metadata = SessionMetadata(
        cwd=WorkingDirectory("/Users/test/project"),
        last_modified=datetime(2024, 1, 1, 12, 0, 0),
        total_messages=42
    )
    return ClaudeSession(
        name=SessionName("-Users-test-project"),
        path=SessionPath("/tmp/test-session"),
        session_count=5,
        metadata=metadata
    )

@pytest.fixture
def sample_sessions() -> List[ClaudeSession]:
    """Create multiple sample sessions for testing"""
    sessions = []
    
    # Aura session
    aura_metadata = SessionMetadata(
        cwd=WorkingDirectory("/Users/test/auras/igris"),
        last_modified=datetime(2024, 1, 1, 10, 0, 0),
        total_messages=100
    )
    aura_session = ClaudeSession(
        name=SessionName("-Users-test-auras-igris"),
        path=SessionPath("/tmp/aura-session"),
        session_count=10,
        metadata=aura_metadata
    )
    sessions.append(aura_session)
    
    # Regular session
    regular_metadata = SessionMetadata(
        cwd=WorkingDirectory("/Users/test/regular-project"),
        last_modified=datetime(2024, 1, 2, 14, 30, 0),
        total_messages=25
    )
    regular_session = ClaudeSession(
        name=SessionName("-Users-test-regular-project"),
        path=SessionPath("/tmp/regular-session"),
        session_count=3,
        metadata=regular_metadata
    )
    sessions.append(regular_session)
    
    return sessions

@pytest.fixture
def mock_cm_output() -> str:
    """Mock output from cm list command"""
    return """Claude Projects and Sessions:

  -Users-test-auras-igris (      10 sessions)
  -Users-test-regular-project (       3 sessions)
  -Users-test-empty-project (       0 sessions)

Total: 3 projects, 13 sessions
"""

# Test classes

class TestResult:
    """Test the Result type"""
    
    def test_success_result(self) -> None:
        """Test successful result"""
        result: Result[str, Exception] = Result(value="success")
        
        assert result.is_success
        assert not result.is_failure
        assert result.unwrap() == "success"
        assert result.unwrap_or("default") == "success"
    
    def test_failure_result(self) -> None:
        """Test failure result"""
        error = ValueError("test error")
        result: Result[str, ValueError] = Result(error=error)
        
        assert not result.is_success
        assert result.is_failure
        assert result.unwrap_or("default") == "default"
        
        with pytest.raises(ValueError, match="test error"):
            result.unwrap()

class TestSessionValidator:
    """Test session validation functions"""
    
    def test_valid_session_name(self) -> None:
        """Test session name validation"""
        assert SessionValidator.is_valid_session_name("-Users-test-project")
        assert SessionValidator.is_valid_session_name("simple-name")
        assert not SessionValidator.is_valid_session_name("")
        assert not SessionValidator.is_valid_session_name("   ")
    
    def test_valid_session_path(self) -> None:
        """Test session path validation"""
        with tempfile.TemporaryDirectory() as temp_dir:
            assert SessionValidator.is_valid_session_path(temp_dir)
            assert not SessionValidator.is_valid_session_path("/nonexistent/path")
            assert not SessionValidator.is_valid_session_path("")
    
    def test_valid_working_directory(self) -> None:
        """Test working directory validation"""
        with tempfile.TemporaryDirectory() as temp_dir:
            assert SessionValidator.is_valid_working_directory(temp_dir)
            assert SessionValidator.is_valid_working_directory("")  # Empty is valid
            assert not SessionValidator.is_valid_working_directory("/nonexistent/path")

class TestSessionMetadata:
    """Test SessionMetadata functionality"""
    
    def test_from_json_data_complete(self) -> None:
        """Test creating metadata from complete JSON data"""
        json_data = {
            "cwd": "/Users/test/project",
            "timestamp": "2024-01-01T12:00:00",
            "message_count": 42
        }
        
        metadata = SessionMetadata.from_json_data(json_data)
        
        assert metadata.cwd == WorkingDirectory("/Users/test/project")
        assert metadata.last_modified == datetime(2024, 1, 1, 12, 0, 0)
        assert metadata.total_messages == 42
    
    def test_from_json_data_minimal(self) -> None:
        """Test creating metadata from minimal JSON data"""
        json_data: Dict[str, Any] = {}
        
        metadata = SessionMetadata.from_json_data(json_data)
        
        assert metadata.cwd is None
        assert metadata.last_modified is None
        assert metadata.total_messages == 0
    
    def test_from_json_data_invalid_timestamp(self) -> None:
        """Test handling invalid timestamps"""
        json_data = {
            "cwd": "/Users/test/project",
            "timestamp": "invalid-timestamp",
            "message_count": 42
        }
        
        metadata = SessionMetadata.from_json_data(json_data)
        
        assert metadata.cwd == WorkingDirectory("/Users/test/project")
        assert metadata.last_modified is None
        assert metadata.total_messages == 42

class TestClaudeSession:
    """Test ClaudeSession functionality"""
    
    def test_display_name(self, sample_session: ClaudeSession) -> None:
        """Test display name formatting"""
        session = ClaudeSession(
            name=SessionName("-Users-tryk-nabia-chats-auras-igris"),
            path=SessionPath("/tmp/test"),
            session_count=5
        )
        
        assert session.display_name == "~/nabia/chats/auras/igris"
    
    def test_current_cwd(self, sample_session: ClaudeSession) -> None:
        """Test current working directory property"""
        assert sample_session.current_cwd == "/Users/test/project"
        
        # Test with no cwd
        session_no_cwd = ClaudeSession(
            name=SessionName("test"),
            path=SessionPath("/tmp/test"),
            session_count=0,
            metadata=SessionMetadata(None, None)
        )
        assert session_no_cwd.current_cwd == ""
    
    def test_is_aura_session(self) -> None:
        """Test aura session detection"""
        aura_session = ClaudeSession(
            name=SessionName("-Users-test-auras-igris"),
            path=SessionPath("/tmp/test"),
            session_count=1
        )
        assert aura_session.is_aura_session()
        
        regular_session = ClaudeSession(
            name=SessionName("-Users-test-regular"),
            path=SessionPath("/tmp/test"),
            session_count=1
        )
        assert not regular_session.is_aura_session()

class TestClaudeManager:
    """Test ClaudeManager functionality"""
    
    def test_init_default_config(self) -> None:
        """Test initialization with default config"""
        manager = ClaudeManager()
        
        assert manager.config.claude_dir == Path.home() / ".claude" / "projects"
        assert manager.config.cm_command == "cm"
        assert manager.sessions == []
    
    def test_init_custom_config(self, test_config: ClaudeManagerConfig) -> None:
        """Test initialization with custom config"""
        manager = ClaudeManager(test_config)
        
        assert manager.config == test_config
        assert manager.sessions == []
    
    @pytest.mark.asyncio
    async def test_run_cm_command_async_success(self, test_config: ClaudeManagerConfig) -> None:
        """Test successful async command execution"""
        manager = ClaudeManager(test_config)
        
        # Mock successful command
        with patch('asyncio.create_subprocess_exec') as mock_subprocess:
            mock_process = AsyncMock()
            mock_process.communicate.return_value = ("success output", "")
            mock_process.returncode = 0
            mock_subprocess.return_value = mock_process
            
            result = await manager.run_cm_command_async("list")
            
            assert result.is_success
            assert result.unwrap() == CommandOutput("success output")
    
    @pytest.mark.asyncio
    async def test_run_cm_command_async_failure(self, test_config: ClaudeManagerConfig) -> None:
        """Test failed async command execution"""
        manager = ClaudeManager(test_config)
        
        # Mock failed command
        with patch('asyncio.create_subprocess_exec') as mock_subprocess:
            mock_process = AsyncMock()
            mock_process.communicate.return_value = ("", "error output")
            mock_process.returncode = 1
            mock_subprocess.return_value = mock_process
            
            result = await manager.run_cm_command_async("list")
            
            assert result.is_failure
            assert isinstance(result.error, CommandExecutionError)
            assert "Command failed with code 1" in str(result.error)
    
    @pytest.mark.asyncio
    async def test_run_cm_command_async_timeout(self, test_config: ClaudeManagerConfig) -> None:
        """Test command timeout"""
        manager = ClaudeManager(test_config)
        
        # Mock timeout
        with patch('asyncio.create_subprocess_exec') as mock_subprocess:
            mock_process = AsyncMock()
            mock_subprocess.return_value = mock_process
            
            with patch('asyncio.wait_for', side_effect=asyncio.TimeoutError):
                result = await manager.run_cm_command_async("list")
                
                assert result.is_failure
                assert isinstance(result.error, CommandExecutionError)
                assert "Command timed out" in str(result.error)
    
    def test_run_cm_command_sync_success(self, test_config: ClaudeManagerConfig) -> None:
        """Test successful sync command execution"""
        manager = ClaudeManager(test_config)
        
        # Mock successful subprocess run
        with patch('subprocess.run') as mock_run:
            mock_result = Mock()
            mock_result.returncode = 0
            mock_result.stdout = "success output"
            mock_result.stderr = ""
            mock_run.return_value = mock_result
            
            result = manager.run_cm_command("list")
            
            assert result.is_success
            assert result.unwrap() == CommandOutput("success output")
    
    @pytest.mark.asyncio
    async def test_parse_session_line_valid(self, test_config: ClaudeManagerConfig) -> None:
        """Test parsing valid session line"""
        manager = ClaudeManager(test_config)
        
        line = "  -Users-test-project (       5 sessions)"
        result = await manager._parse_session_line(line)
        
        assert result.is_success
        session = result.unwrap()
        assert session.name == SessionName("-Users-test-project")
        assert session.session_count == 5
    
    @pytest.mark.asyncio
    async def test_parse_session_line_invalid(self, test_config: ClaudeManagerConfig) -> None:
        """Test parsing invalid session line"""
        manager = ClaudeManager(test_config)
        
        line = "invalid line format"
        result = await manager._parse_session_line(line)
        
        assert result.is_failure
        assert isinstance(result.error, SessionLoadError)
    
    def test_get_session_by_name(self, test_config: ClaudeManagerConfig, sample_sessions: List[ClaudeSession]) -> None:
        """Test getting session by name"""
        manager = ClaudeManager(test_config)
        manager.sessions = sample_sessions
        manager._update_session_cache()
        
        session = manager.get_session_by_name(SessionName("-Users-test-auras-igris"))
        assert session is not None
        assert session.name == SessionName("-Users-test-auras-igris")
        
        session = manager.get_session_by_name(SessionName("nonexistent"))
        assert session is None

class TestSearchResult:
    """Test SearchResult functionality"""
    
    def test_comparison(self, sample_sessions: List[ClaudeSession]) -> None:
        """Test search result comparison"""
        result1 = SearchResult(sample_sessions[0], 10, ["match1"])
        result2 = SearchResult(sample_sessions[1], 5, ["match2"])
        result3 = SearchResult(sample_sessions[0], 10, ["match3"])
        
        assert result1 > result2
        assert result2 < result1
        assert result1 == result3

class TestSearchFunctionality:
    """Test search functionality"""
    
    def test_search_sessions_basic(self, test_config: ClaudeManagerConfig, sample_sessions: List[ClaudeSession]) -> None:
        """Test basic session search"""
        manager = ClaudeManager(test_config)
        manager.sessions = sample_sessions
        
        # Search for "auras"
        results = manager.search_sessions(SearchQuery("auras"))
        
        assert len(results) == 1
        assert results[0].session.name == SessionName("-Users-test-auras-igris")
        assert results[0].score > 0
    
    def test_search_sessions_multiple_keywords(self, test_config: ClaudeManagerConfig, sample_sessions: List[ClaudeSession]) -> None:
        """Test search with multiple keywords"""
        manager = ClaudeManager(test_config)
        manager.sessions = sample_sessions
        
        # Search for "test project"
        results = manager.search_sessions(SearchQuery("test project"))
        
        assert len(results) >= 1
        # Should find sessions containing either "test" or "project"
        
    def test_search_sessions_empty_query(self, test_config: ClaudeManagerConfig, sample_sessions: List[ClaudeSession]) -> None:
        """Test search with empty query"""
        manager = ClaudeManager(test_config)
        manager.sessions = sample_sessions
        
        results = manager.search_sessions(SearchQuery(""))
        assert len(results) == 0
        
        results = manager.search_sessions(SearchQuery("   "))
        assert len(results) == 0
    
    def test_search_sessions_no_matches(self, test_config: ClaudeManagerConfig, sample_sessions: List[ClaudeSession]) -> None:
        """Test search with no matches"""
        manager = ClaudeManager(test_config)
        manager.sessions = sample_sessions
        
        results = manager.search_sessions(SearchQuery("nonexistent"))
        assert len(results) == 0

# Integration tests

@pytest.mark.asyncio
async def test_full_session_loading_workflow(test_config: ClaudeManagerConfig, mock_cm_output: str) -> None:
    """Test the complete session loading workflow"""
    manager = ClaudeManager(test_config)
    
    # Mock the cm list command
    with patch.object(manager, 'run_cm_command_async') as mock_cmd:
        mock_cmd.return_value = Result(value=CommandOutput(mock_cm_output))
        
        # Mock metadata loading
        with patch.object(manager, 'load_session_metadata') as mock_metadata:
            test_metadata = SessionMetadata(
                cwd=WorkingDirectory("/Users/test/project"),
                last_modified=datetime.now(),
                total_messages=10
            )
            mock_metadata.return_value = Result(value=test_metadata)
            
            result = await manager.load_sessions_async()
            
            assert result.is_success
            sessions = result.unwrap()
            assert len(sessions) >= 2  # Should have loaded multiple sessions
            
            # Check that sessions were cached
            assert len(manager._session_cache) == len(sessions)

# Performance tests

@pytest.mark.benchmark
def test_session_search_performance(test_config: ClaudeManagerConfig) -> None:
    """Benchmark session search performance"""
    manager = ClaudeManager(test_config)
    
    # Create many test sessions
    sessions = []
    for i in range(1000):
        metadata = SessionMetadata(
            cwd=WorkingDirectory(f"/Users/test/project-{i}"),
            last_modified=datetime.now(),
            total_messages=i
        )
        session = ClaudeSession(
            name=SessionName(f"-Users-test-project-{i}"),
            path=SessionPath(f"/tmp/session-{i}"),
            session_count=i % 10,
            metadata=metadata
        )
        sessions.append(session)
    
    manager.sessions = sessions
    
    # Benchmark search
    import time
    start_time = time.time()
    
    results = manager.search_sessions(SearchQuery("project test"))
    
    end_time = time.time()
    search_time = end_time - start_time
    
    # Should complete search in reasonable time
    assert search_time < 1.0  # Less than 1 second
    assert len(results) > 0  # Should find matches

# Parametrized tests

@pytest.mark.parametrize("session_line,expected_name,expected_count", [
    ("  -Users-test-project (       5 sessions)", "-Users-test-project", 5),
    ("  -Users-another-project (      10 sessions)", "-Users-another-project", 10),
    ("  simple-name (       1 session)", "simple-name", 1),
])
@pytest.mark.asyncio
async def test_parse_session_lines_parametrized(
    test_config: ClaudeManagerConfig,
    session_line: str,
    expected_name: str,
    expected_count: int
) -> None:
    """Test parsing various session line formats"""
    manager = ClaudeManager(test_config)
    
    result = await manager._parse_session_line(session_line)
    
    assert result.is_success
    session = result.unwrap()
    assert session.name == SessionName(expected_name)
    assert session.session_count == expected_count

# Error handling tests

@pytest.mark.asyncio
async def test_error_handling_chain(test_config: ClaudeManagerConfig) -> None:
    """Test error handling through the entire chain"""
    manager = ClaudeManager(test_config)
    
    # Mock command failure
    with patch.object(manager, 'run_cm_command_async') as mock_cmd:
        mock_cmd.return_value = Result(error=CommandExecutionError("Command failed"))
        
        result = await manager.load_sessions_async()
        
        assert result.is_failure
        assert isinstance(result.error, SessionLoadError)
        assert "Failed to list sessions" in str(result.error)

# Type safety tests

def test_type_safety_session_validator() -> None:
    """Test that type validators work correctly with type checker"""
    # These tests verify TypeIs functionality
    test_name = "valid-session-name"
    if SessionValidator.is_valid_session_name(test_name):
        # Type checker should now know test_name is SessionName
        session_name: SessionName = test_name  # This should not cause type errors
        assert isinstance(session_name, str)
    
    test_path = "/tmp"
    if SessionValidator.is_valid_session_path(test_path):
        # Type checker should now know test_path is SessionPath
        session_path: SessionPath = test_path  # This should not cause type errors
        assert isinstance(session_path, str)

if __name__ == "__main__":
    # Run tests with coverage
    pytest.main([
        __file__,
        "-v",
        "--cov=claude_manager_tui_typed",
        "--cov-report=html",
        "--cov-report=term-missing",
        "--benchmark-only",
        "--benchmark-sort=mean"
    ])
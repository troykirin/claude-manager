/**
 * TypeScript type definitions mirroring the Rust models for federation integration
 */

export interface Session {
  id: string;
  metadata: SessionMetadata;
  blocks: Block[];
  insights: SessionInsights;
  statistics: SessionStatistics;
  tool_usage: ToolUsageStats;
  working_context: WorkingContext;
}

export interface SessionMetadata {
  file_path: string;
  created_at: string;
  last_modified: string;
  file_size_bytes: number;
  line_count: number;
  claude_version?: string;
  client_info?: ClientInfo;
  session_duration?: number; // milliseconds
  conversation_id?: string;
  project_context?: ProjectContext;
}

export interface ClientInfo {
  name: string;
  version: string;
  platform: string;
  user_agent?: string;
}

export interface ProjectContext {
  working_directory?: string;
  project_name?: string;
  project_type?: ProjectType;
  language_stack: ProgrammingLanguage[];
  frameworks: string[];
  repository_url?: string;
  git_branch?: string;
}

export enum ProjectType {
  WebApp = 'WebApp',
  MobileApp = 'MobileApp',
  Library = 'Library',
  CLI = 'CLI',
  DataScience = 'DataScience',
  MachineLearning = 'MachineLearning',
  Documentation = 'Documentation',
  Configuration = 'Configuration',
  Unknown = 'Unknown',
}

export enum ProgrammingLanguage {
  Rust = 'Rust',
  Python = 'Python',
  JavaScript = 'JavaScript',
  TypeScript = 'TypeScript',
  Java = 'Java',
  Go = 'Go',
  Cpp = 'Cpp',
  C = 'C',
  Swift = 'Swift',
  Kotlin = 'Kotlin',
  Ruby = 'Ruby',
  PHP = 'PHP',
  Dart = 'Dart',
  Shell = 'Shell',
  SQL = 'SQL',
  HTML = 'HTML',
  CSS = 'CSS',
  Markdown = 'Markdown',
  JSON = 'JSON',
  YAML = 'YAML',
  TOML = 'TOML',
}

export interface Block {
  id: string;
  sequence_number: number;
  role: Role;
  timestamp: string;
  content: BlockContent;
  metadata: BlockMetadata;
  tools: ToolInvocation[];
  attachments: Attachment[];
  context_references: ContextReference[];
}

export enum Role {
  User = 'User',
  Assistant = 'Assistant',
  System = 'System',
  Tool = 'Tool',
}

export interface BlockContent {
  raw_text: string;
  formatted_text?: string;
  tokens: ContentToken[];
  code_blocks: CodeBlock[];
  links: Link[];
  mentions: Mention[];
  word_count: number;
  character_count: number;
}

export interface ContentToken {
  text: string;
  token_type: TokenType;
  position: number;
  length: number;
}

export enum TokenType {
  Word = 'Word',
  Number = 'Number',
  Punctuation = 'Punctuation',
  Code = 'Code',
  FilePath = 'FilePath',
  URL = 'URL',
  Command = 'Command',
  Variable = 'Variable',
  Function = 'Function',
  Class = 'Class',
  Method = 'Method',
  Keyword = 'Keyword',
  String = 'String',
  Comment = 'Comment',
}

export interface CodeBlock {
  language?: ProgrammingLanguage;
  content: string;
  line_numbers: boolean;
  filename?: string;
  start_position: number;
  end_position: number;
}

export interface Link {
  url: string;
  title?: string;
  link_type: LinkType;
}

export enum LinkType {
  External = 'External',
  Documentation = 'Documentation',
  Repository = 'Repository',
  File = 'File',
  Internal = 'Internal',
}

export interface Mention {
  text: string;
  mention_type: MentionType;
  context?: string;
}

export enum MentionType {
  File = 'File',
  Function = 'Function',
  Class = 'Class',
  Variable = 'Variable',
  Command = 'Command',
  Person = 'Person',
  Project = 'Project',
  Library = 'Library',
  Tool = 'Tool',
}

export interface BlockMetadata {
  processing_time_ms?: number;
  confidence_score?: number;
  complexity_score?: number;
  sentiment?: Sentiment;
  topics: string[];
  intent?: ConversationIntent;
  parent_block_id?: string;
  thread_id?: string;
}

export enum Sentiment {
  Positive = 'Positive',
  Negative = 'Negative',
  Neutral = 'Neutral',
  Mixed = 'Mixed',
}

export enum ConversationIntent {
  Question = 'Question',
  Request = 'Request',
  Explanation = 'Explanation',
  Debugging = 'Debugging',
  CodeReview = 'CodeReview',
  Planning = 'Planning',
  Learning = 'Learning',
  Troubleshooting = 'Troubleshooting',
  Documentation = 'Documentation',
  Implementation = 'Implementation',
}

export interface ToolInvocation {
  tool_name: string;
  parameters: Record<string, any>;
  result?: ToolResult;
  timestamp: string;
  execution_time_ms?: number;
  success: boolean;
  error_message?: string;
}

export interface ToolResult {
  content: string;
  result_type: ToolResultType;
  metadata?: Record<string, any>;
  files_affected: string[];
}

export enum ToolResultType {
  Success = 'Success',
  Error = 'Error',
  Warning = 'Warning',
  Information = 'Information',
  FileContent = 'FileContent',
  CommandOutput = 'CommandOutput',
  SearchResults = 'SearchResults',
}

export interface Attachment {
  filename: string;
  file_type: AttachmentType;
  size_bytes?: number;
  content?: string;
  hash?: string;
}

export enum AttachmentType {
  Image = 'Image',
  Document = 'Document',
  Code = 'Code',
  Data = 'Data',
  Archive = 'Archive',
  Other = 'Other',
}

export interface ContextReference {
  reference_type: ReferenceType;
  target_block_id: string;
  relevance_score: number;
  description?: string;
}

export enum ReferenceType {
  Continuation = 'Continuation',
  Response = 'Response',
  Clarification = 'Clarification',
  Example = 'Example',
  Alternative = 'Alternative',
  Correction = 'Correction',
}

export interface WorkingContext {
  files_mentioned: Record<string, FileContext>;
  commands_run: CommandExecution[];
  directories_accessed: string[];
  tools_used: string[];
  error_patterns: ErrorPattern[];
  solution_patterns: SolutionPattern[];
}

export interface FileContext {
  path: string;
  file_type?: ProgrammingLanguage;
  mentions: number;
  operations: FileOperation[];
  last_accessed: string;
}

export enum FileOperation {
  Read = 'Read',
  Write = 'Write',
  Create = 'Create',
  Delete = 'Delete',
  Move = 'Move',
  Copy = 'Copy',
  Edit = 'Edit',
  Search = 'Search',
}

export interface CommandExecution {
  command: string;
  working_directory?: string;
  timestamp: string;
  exit_code?: number;
  output?: string;
  duration_ms?: number;
}

export interface ErrorPattern {
  error_type: string;
  pattern: string;
  occurrences: number;
  resolution_attempts: string[];
  resolved: boolean;
}

export interface SolutionPattern {
  problem_type: string;
  solution_approach: string;
  tools_used: string[];
  success_rate: number;
  context: string;
}

export interface SessionInsights {
  primary_topics: Topic[];
  conversation_flow: ConversationFlow;
  learning_outcomes: LearningOutcome[];
  productivity_metrics: ProductivityMetrics;
  collaboration_patterns: CollaborationPatterns;
}

export interface Topic {
  name: string;
  relevance_score: number;
  mentions: number;
  subtopics: string[];
  related_tools: string[];
}

export interface ConversationFlow {
  phases: ConversationPhase[];
  transitions: PhaseTransition[];
  complexity_evolution: number[];
  focus_shifts: number;
}

export interface ConversationPhase {
  phase_type: PhaseType;
  start_block: number;
  end_block: number;
  duration: number; // milliseconds
  primary_activity: string;
}

export enum PhaseType {
  Planning = 'Planning',
  Implementation = 'Implementation',
  Debugging = 'Debugging',
  Testing = 'Testing',
  Documentation = 'Documentation',
  Learning = 'Learning',
  Review = 'Review',
}

export interface PhaseTransition {
  from_phase: PhaseType;
  to_phase: PhaseType;
  trigger: string;
  block_number: number;
}

export interface LearningOutcome {
  skill_area: string;
  concepts_learned: string[];
  complexity_level: ComplexityLevel;
  confidence_gain: number;
  practical_application: boolean;
}

export enum ComplexityLevel {
  Beginner = 'Beginner',
  Intermediate = 'Intermediate',
  Advanced = 'Advanced',
  Expert = 'Expert',
}

export interface ProductivityMetrics {
  tasks_completed: number;
  problems_solved: number;
  code_quality_score: number;
  efficiency_rating: number;
  collaboration_effectiveness: number;
  time_to_resolution: number[]; // milliseconds
}

export interface CollaborationPatterns {
  interaction_style: InteractionStyle;
  question_types: Record<string, number>;
  feedback_quality: number;
  iterative_cycles: number;
  knowledge_transfer: number;
}

export enum InteractionStyle {
  Exploratory = 'Exploratory',
  TaskOriented = 'TaskOriented',
  Learning = 'Learning',
  Debugging = 'Debugging',
  Creative = 'Creative',
  Analytical = 'Analytical',
}

export interface SessionStatistics {
  total_blocks: number;
  user_blocks: number;
  assistant_blocks: number;
  tool_blocks: number;
  total_words: number;
  total_characters: number;
  code_blocks: number;
  files_referenced: number;
  commands_executed: number;
  errors_encountered: number;
  session_duration?: number; // milliseconds
  average_response_time?: number; // milliseconds
}

export interface ToolUsageStats {
  tools_by_frequency: Record<string, number>;
  total_tool_calls: number;
  successful_calls: number;
  failed_calls: number;
  average_execution_time: number;
  most_used_tool?: string;
  tool_efficiency: Record<string, number>;
}

// Federation-specific types
export interface Thread {
  id: string;
  blocks: Block[];
  metadata: ThreadMetadata;
  context: ThreadContext;
  assignments: Record<AgentRole, ThreadAssignment>;
}

export interface ThreadMetadata {
  origin: string;
  project: string;
  timestamp: string;
  priority: Priority;
  estimated_complexity: ComplexityLevel;
}

export interface ThreadContext {
  working_directory?: string;
  technologies: string[];
  domain: string;
  objectives: string[];
  constraints: string[];
}

export interface ThreadAssignment {
  agent_id: string;
  assigned_at: string;
  status: AssignmentStatus;
  progress?: number;
  deliverables: string[];
}

export enum AgentRole {
  Orchestrator = 'orchestrator',
  Worker = 'worker',
  Analyst = 'analyst',
  Architect = 'architect',
  Debugger = 'debugger',
  Implementer = 'implementer',
}

export enum AssignmentStatus {
  Pending = 'pending',
  InProgress = 'in_progress',
  Completed = 'completed',
  Failed = 'failed',
  Cancelled = 'cancelled',
}

export enum Priority {
  Low = 'low',
  Medium = 'medium',
  High = 'high',
  Urgent = 'urgent',
}

export interface Task {
  id: string;
  title: string;
  description: string;
  type: TaskType;
  priority: Priority;
  assignee?: string;
  project?: string;
  labels: string[];
  due_date?: string;
  created_from: {
    session_id: string;
    block_id: string;
    marker_type: string;
  };
}

export enum TaskType {
  Implementation = 'implementation',
  Bug = 'bug',
  Enhancement = 'enhancement',
  Research = 'research',
  Documentation = 'documentation',
  Testing = 'testing',
}

export interface Marker {
  id: string;
  block_id: string;
  type: MarkerType;
  content: string;
  context: string;
  confidence: number;
  tags: string[];
}

export enum MarkerType {
  ActionItem = 'action_item',
  Question = 'question',
  Decision = 'decision',
  Issue = 'issue',
  Insight = 'insight',
  TODO = 'todo',
  FIXME = 'fixme',
  NOTE = 'note',
}

export interface Insight {
  id: string;
  title: string;
  content: string;
  category: InsightCategory;
  confidence: number;
  related_entities: string[];
  session_context: {
    session_id: string;
    blocks: string[];
    timestamp: string;
  };
}

export enum InsightCategory {
  Technical = 'technical',
  Process = 'process',
  Learning = 'learning',
  Pattern = 'pattern',
  Decision = 'decision',
  Risk = 'risk',
}
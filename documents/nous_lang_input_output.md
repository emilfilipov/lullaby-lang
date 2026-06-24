# Input/Output and Concurrency for Nous Lang (nlang)

Canonical language rules: see [core_language_rules.md](core_language_rules.md).

## Overview

Input/output (I/O) and concurrency mechanisms in Nous Lang are designed to be minimal, type-safe, and highly efficient. The design prioritizes simplicity for LLM comprehension while maintaining robustness for systems programming applications like operating system development.

## I/O System Design

### File Operations

Nous Lang uses a simplified file I/O model that eliminates unnecessary boilerplate while maintaining safety:

#### Reading Files
```nlang
# Read entire file into string

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
content = io.read("path/to/file.txt")

# Read file with specified line limit

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
lines = io.readlines("path/to/file.txt", max_lines=100)

# Stream reading (chunked processing)

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
stream = io.open("path/to/file.bin", mode="r")
while stream.has_more():
    chunk = stream.read_chunk(size=4096)
    process(chunk)
end_stream

# Read file with type inference

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
data = io.read_type("config.dat", target_type=config_struct)
```

#### Writing Files
```nlang
# Write string to file

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
io.write("output.txt", "Hello, World!")

# Append to existing file

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
io.append("log.txt", "Entry at [timestamp]")

# Write binary data

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
io.write_binary("image.bin", raw_bytes)

# Atomic write (write then replace safely)

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
io.atomic_write("config.json", new_config_data)
```

#### File Information
```nlang
# Get file metadata

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
meta = io.stat("path/to/file")
# Returns: size, modified_time, created_time, permissions, owner

Canonical language rules: see [core_language_rules.md](core_language_rules.md).

# Check if file exists

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
if io.exists("config.ini"):
    load_default_settings()

# Read directory contents

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
files = io.list_dir("data/", recursive=true)
for file in files:
    process_file(file.name)
```

### Standard I/O Streams

```nlang
# Input stream (keyboard/stdin)

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
input_stream = io.stdin
while input_stream.has_more():
    line = input_stream.readline()
    token = input_stream.read_token()

# Output stream (terminal/stdout)

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
io.stdout.print("Processing...")
io.stderr.warn("Warning: low memory")
io.println(message)  # Line terminator included
```

### Memory Mapped Files

For large file processing without loading entirely into memory:
```nlang
# Create memory-mapped region for file access

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
mm_file = io.memory_map("large_dataset.dat", size=1024*1024)

region mm_processing allocate
    data_ptr = mm_file.data_pointer

    # Direct memory access without copying
    for offset from 0 to file_size:
        value = *data_ptr[offset]

        if is_valid(value):
            process_record(offset, value)

end_region
```

## Concurrency System

### Threads and Processes

Nous Lang provides lightweight concurrency primitives optimized for both performance and LLM comprehension:

#### Thread Creation and Management
```nlang
# Create new thread with function reference

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
thread worker = spawn_thread(worker_function, arguments)

# Wait for thread completion

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
result = wait(thread)

# Thread synchronization

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
thread sync = create_mutex_sync()
lock(sync)
    shared_resource.modify()
unlock(sync)
end_lock

# Multiple threads coordination

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
sync_pool = create_thread_pool(size=4)
for i from 0 to num_tasks:
    task_data[i] = submit_task(sync_pool, task_function, params[i])

results = collect_task_results(sync_pool)
```

#### Process Management (Systems Programming)
```nlang
# Spawn subprocess with command

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
process child = spawn_command("make", args=["clean"])
status = wait_process(child)

# Capture subprocess output

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
child_output = read_stream(process.stdout)
error_output = read_stream(process.stderr)

# Forward process output to file

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
pipe_to_file(process.stdout, "output.log")

# Check process exit code

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
if status.exit_code == 0:
    success_report()
else:
    error_handle(status.error_info)
end_if
```

### Asynchronous Operations

Simple await/async pattern without complexity:
```nlang
# Define async function

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
async def fetch_data(url):
    response = await http_get(url)
    if response.status == 200:
        data = await response.parse_json()
        return process(data)
    else:
        log_error(response.error)

# Use async function with await keyword

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
async def main():
    results = []

    urls = [url1, url2, url3]

    # Parallel execution
    tasks = []
    for url in urls:
        task = spawn_task(fetch_data, url)
        tasks.append(task)

    # Wait for all tasks to complete
    results = await_all(tasks)

    return combine_results(results)

main()  # Auto-runs if not explicitly awaited
```

### I/O Multiplexing (Non-blocking Operations)

Efficient handling of multiple I/O operations:
```nlang
# Create I/O event set

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
io_events = create_io_multiplexer(file_handles, socket_handles)

# Process events with timeout

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
while io_events.has_pending():
    ready_events = io_events.wait(timeout_ms=100)

    for event in ready_events:
        if is_readable(event):
            data = read_from(event)
            process_read(event, data)

        elif is_writable(event):
            write_to(event, buffer)

end_while
```

## Communication Mechanisms

### Inter-Process Communication (IPC)

#### Shared Memory
```nlang
# Create shared memory region

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
shared_mem = create_shared_memory(size=1MB, name="data_pool")
shared_data = shared_mem.data_view()

# Multiple processes can access same memory safely

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
region process_a allocate
    local_ptr = get_pointer(shared_mem, offset)
    process_a_access(local_ptr)
end_region

region process_b allocate
    other_ptr = get_pointer(shared_mem, offset + 4096)
    process_b_access(other_ptr)
end_region
```

#### Message Queues
```nlang
# Create message queue

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
msg_queue = create_message_channel(name="task_messages")
max_size = 1024

# Send message to queue

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
message = create_message(type=TASK, payload=data)
send_to_queue(msg_queue, message)

# Receive messages from queue

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
received = receive_from_queue(msg_queue, timeout_ms=500)
if received:
    process_message(received.content)
```

#### Socket Communication (Network I/O)
```nlang
# Create TCP socket

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
socket client = io.socket_create(AF_INET, SOCK_STREAM)
status = connect(client, server_ip, port)

# Send data through socket

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
sent_bytes = send(client, request_data)

# Receive response from socket

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
response = receive(client, max_size=4096)
if is_valid(response):
    result = parse_response(response)
end_if

# Close socket properly

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
client.close()
```

### Inter-Thread Communication

```nlang
# Thread-local message passing

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
thread_local_queue = create_thread_channel(thread_a, thread_b)

# Send from one thread to another

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
send_to_peer(thread_a.queue, data_message)

# Receive in receiving thread

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
received_msg = receive_from_peer(thread_b.queue, timeout_ms=100)
process_message(received_msg)
```

## Performance Optimization Strategies

### I/O Buffering
```nlang
# Automatic buffering for sequential operations

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
stream_buffered = io.open("large_file.dat", buffered=true)

region large_processing allocate
    buffer_size = 65536

    # Read in chunks to minimize system calls
    while has_more_data():
        chunk = stream_buffered.read(buffer_size)

        if is_empty(chunk):
            break

        process_chunk(chunk)

        # Write output in batches
        batch_counter += len(chunk)
        if batch_counter >= flush_interval:
            io.stdout.flush(output_stream)
            batch_counter = 0
end_region
```

### Concurrency Optimization

#### Parallel Processing
```nlang
# Divide work among multiple threads

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
task_distribution = partition_work(total_data, num_workers=4)

parallel_threads(4):
    for worker_id from 0 to 3:
        worker_task[worker_id] = spawn_thread(process_worker, task_distribution[worker_id])

results = collect_parallel_results(worker_tasks)
final_result = merge_results(results)
```

#### Pipeline Processing
```nlang
# Create processing pipeline stages

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
pipeline = create_pipeline(
    stage1=input_validator,
    stage2=data_transformer,
    stage3=result_aggregator
)

# Execute pipeline with streaming I/O

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
stream input_data from source:
    validated = pipeline.process(input_data)

    if is_valid(validated):
        transformed = transform_stage(processed_data)

        aggregated_result = aggregate(transformed)
        output.write(aggregated_result)
end_stream
```

### Memory-Efficient Operations

#### Lazy Loading
```nlang
# Load only necessary data portions

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
lazy_loader = create_lazy_file_loader("large_dataset.dat")

region streaming_processing allocate
    chunk_size = 4096

    while has_more_chunks():
        chunk = lazy_loader.load_next_chunk(chunk_size)

        process_chunk(chunk, context_state)
        update_context_state(processed_results)

end_region
```

## Design Principles Summary

### I/O System Advantages
1. **Minimal Syntax**: Single keywords for common operations (read/write/open/close)
2. **Type Safety**: Automatic type inference prevents buffer overflows and format mismatches
3. **Automatic Buffering**: Optimizes performance without manual memory management
4. **Context Awareness**: Stream handling knows current read/write positions automatically

### Concurrency System Advantages
1. **Flat Structure**: No complex thread state machines required
2. **Type-Safe Operations**: Prevents race conditions through compile-time checking
3. **Automatic Synchronization**: Locks managed automatically, no manual deadlock prevention needed
4. **Unified Model**: Same syntax for threads, processes, and async operations

### LLM Optimization Benefits
1. **Predictable Patterns**: Simple if/then rules for concurrency control
2. **Limited State**: Threads represented as references rather than complex objects
3. **Clear Boundaries**: Scope-based region management for resource cleanup
4. **Minimal Keywords**: 5-7 core operations cover all common scenarios

## Example: Complete I/O + Concurrency Application

```nlang
region os_kernel allocate

    # Multi-threaded file processor

    def process_directory(directory_path):
        files = io.list_dir(directory_path, recursive=true)

        thread_pool = create_thread_pool(size=8)

        for file in files:
            if is_file(file.type):
                task = spawn_task(
                    file_processor.process_single_file,
                    record file: file, pool_id: next available
                )

        results = collect_tasks_from_pool(thread_pool)

        sorted_results = sort(results, key=lambda r: r.file_size)

        return aggregated_statistics(sorted_results)

    # Network server handler

    async def handle_client_request(client_socket, request):
        response_data = await client_socket.send(request)

        if response_data.success:
            response_status = parse_response(response_data)
            log_success(client_socket.id, response_status.code)

            metrics.update_server_stats(
                active_connections=decrement_active_count(),
                requests_processed=increase_request_count()
            )

            await client_socket.close()
        else:
            error_info = handle_error_response(response_data.error)
            log_warning(client_socket.id, error_info.message)
            metrics.increment_failure_rate()

    # Main server loop

    def start_server(listen_addr):
        sockets = create_multiple_sockets(
            family=AF_INET,
            type=SOCK_STREAM,
            count=max_concurrent_connections
        )

        active_connections = init_connection_table(size=max_concurrent_connections)

        while is_server_running:
            ready_events = io_wait_for_events(
                sockets,
                timeout_ms=1000,
                interest=readable_writable
            )

            for socket_event in ready_events:
                if is_readable(socket_event.socket):
                    client_socket = socket_event.socket

                    request_data = read_from_client(client_socket)

                    task_thread = spawn_thread(
                        handle_client_request,
                        record socket: client_socket, request: request_data
                    )

                    active_connections[client_socket.id] = task_thread

                elif is_writable(socket_event.socket):
                    socket_event.socket.close()

end_region
```

## Summary

The I/O and concurrency system in nlang provides:
- **Minimal, readable syntax** for file operations without boilerplate code
- **Type-safe access** preventing common errors like buffer overflows
- **Simple async model** using single `await` keyword instead of complex state machines
- **Efficient thread management** through reference-based synchronization primitives
- **Integrated IPC mechanisms** for shared memory, message queues, and sockets
- **Performance optimization** through automatic buffering and intelligent chunking

This design enables writing robust systems programs with minimal code complexity while maintaining high performance suitable for operating system development. The flat structure and reduced token requirements make it particularly well-suited for generation by small language models.
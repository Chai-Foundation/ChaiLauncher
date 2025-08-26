/// Generate Dockerfile for Minecraft server
pub fn generate_dockerfile(java_version: &str) -> String {
    format!(r#"FROM openjdk:{}-alpine

# Install necessary tools
RUN apk add --no-cache curl wget

# Create minecraft user and directory
RUN addgroup -g 1000 minecraft && \
    adduser -D -s /bin/sh -u 1000 -G minecraft minecraft

# Create minecraft directory
RUN mkdir -p /minecraft && \
    mkdir -p /logs && \
    chown -R minecraft:minecraft /minecraft /logs

# Switch to minecraft user
USER minecraft
WORKDIR /minecraft

# Default server port
EXPOSE 25565

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
  CMD ps aux | grep -q java || exit 1

# Start script will be provided at runtime
CMD ["sh", "-c", "java $JAVA_OPTS -jar server.jar nogui"]
"#, java_version)
}

/// Generate server startup script
pub fn generate_startup_script(
    memory_limit: u64,
    max_players: u32,
    server_name: &str,
    additional_args: &[String],
) -> String {
    let mut java_args = vec![
        format!("-Xmx{}M", memory_limit),
        format!("-Xms{}M", memory_limit / 2),
        "-XX:+UseG1GC".to_string(),
        "-XX:+ParallelRefProcEnabled".to_string(),
        "-XX:MaxGCPauseMillis=200".to_string(),
        "-XX:+UnlockExperimentalVMOptions".to_string(),
        "-XX:+DisableExplicitGC".to_string(),
        "-XX:+AlwaysPreTouch".to_string(),
        "-XX:G1NewSizePercent=30".to_string(),
        "-XX:G1MaxNewSizePercent=40".to_string(),
        "-XX:G1HeapRegionSize=8M".to_string(),
        "-XX:G1ReservePercent=20".to_string(),
        "-XX:G1HeapWastePercent=5".to_string(),
        "-XX:G1MixedGCCountTarget=4".to_string(),
        "-XX:InitiatingHeapOccupancyPercent=15".to_string(),
        "-XX:G1MixedGCLiveThresholdPercent=90".to_string(),
        "-XX:G1RSetUpdatingPauseTimePercent=5".to_string(),
        "-XX:SurvivorRatio=32".to_string(),
        "-XX:+PerfDisableSharedMem".to_string(),
        "-XX:MaxTenuringThreshold=1".to_string(),
    ];

    java_args.extend(additional_args.iter().cloned());

    format!(r#"#!/bin/sh
set -e

# Create server.properties if it doesn't exist
if [ ! -f server.properties ]; then
    cat > server.properties << EOF
server-name={server_name}
gamemode=survival
difficulty=easy
spawn-protection=16
max-players={max_players}
level-name=world
level-type=minecraft\:normal
allow-flight=false
announce-player-achievements=true
server-port=25565
online-mode=true
pvp=true
enable-command-block=false
hardcore=false
snooper-enabled=true
max-world-size=29999984
enable-query=false
enable-rcon=false
motd=A Minecraft Server powered by ChaiLauncher
EOF
fi

# Ensure EULA is accepted
echo "eula=true" > eula.txt

# Start the server with optimized JVM settings
echo "Starting Minecraft server with {memory} MB of RAM..."
echo "Server name: {server_name}"
echo "Max players: {max_players}"

exec java {java_args} -jar server.jar nogui
"#,
        server_name = server_name,
        max_players = max_players,
        memory = memory_limit,
        java_args = java_args.join(" ")
    )
}

/// Generate docker-compose.yml for easier server management
pub fn generate_docker_compose(
    server_name: &str,
    image_name: &str,
    port: u16,
    memory_limit: u64,
    game_dir: &str,
    environment_vars: &std::collections::HashMap<String, String>,
) -> String {
    let mut env_vars = vec![
        format!("EULA=TRUE"),
        format!("MAX_PLAYERS={}", environment_vars.get("MAX_PLAYERS").unwrap_or(&"20".to_string())),
        format!("SERVER_NAME={}", server_name),
        format!("JAVA_OPTS=-Xmx{}M -Xms{}M", memory_limit, memory_limit / 2),
    ];

    for (key, value) in environment_vars {
        if !key.starts_with("MAX_PLAYERS") && !key.starts_with("SERVER_NAME") && !key.starts_with("EULA") {
            env_vars.push(format!("{}={}", key, value));
        }
    }

    format!(r#"version: '3.8'

services:
  {service_name}:
    image: {image_name}
    container_name: chai-server-{server_name}
    restart: unless-stopped
    ports:
      - "{port}:25565"
    volumes:
      - "{game_dir}:/minecraft"
      - "chai-server-logs-{server_name}:/logs"
    environment:
{env_vars}
    mem_limit: {memory_limit}m
    healthcheck:
      test: ["CMD", "ps", "aux", "|", "grep", "-q", "java"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 60s

volumes:
  chai-server-logs-{server_name}:
    external: false
"#,
        service_name = server_name.replace(" ", "_").to_lowercase(),
        image_name = image_name,
        server_name = server_name,
        port = port,
        game_dir = game_dir,
        memory_limit = memory_limit,
        env_vars = env_vars.iter().map(|var| format!("      - {}", var)).collect::<Vec<_>>().join("\n")
    )
}
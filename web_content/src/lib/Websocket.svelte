<script lang="ts">
  import Hash from './Hash.svelte';

  let connection_status: string = "Disconnected"
	var socket: WebSocket | null = null
	var hashes: Map<string, Map<string, string>> = new Map;
	let request_obj = new Object;
	request_obj["request"] = [
		"test:1",
		"test:2"
	];

	function connect() {
		disconnect()

		const { location } = window

		const proto = location.protocol.startsWith('https') ? 'wss' : 'ws'
		
		const wsUri = `${proto}://${location.hostname}:8080/ws`

		socket = new WebSocket(wsUri)



		socket.onopen = () => {
			connection_status = "Connected";
			socket.send(JSON.stringify(request_obj))
		}

		socket.onmessage = (ev) => {
			let message = JSON.parse(
				ev.data,
				(key, value) => {
					if(key == "upsert") {
						return new Map(Object.entries(value));
					}
					return value;
				}
			)

			if (!(message.name in hashes)) {
				hashes.set(message.name, new Map);
			}

			message.upsert.forEach((value, key, map) => {
				hashes.get(message.name).set(key, value);
			});
			message.delete.forEach(key => {
				hashes.get(message.name).delete(key);
			});

			hashes = hashes;
		}

		socket.onclose = () => {
			hashes.clear()
			socket = null
			connection_status = "Disconnected";
		}
	}

	function disconnect() {
		if (socket) {
			socket.close()
		}
	}

	function toggle_connection() {
		if (socket) {
			disconnect()
		} else {
			connect()
		}
	}

	connect()
</script>

<div>

	<button on:click={toggle_connection}>
		Status: {connection_status}
	</button>
	
	{#each [...hashes] as hash (hash[0])}
	<Hash 
	name={hash[0]}
	content={hash[1]}
	/>
	{/each}
	
</div>
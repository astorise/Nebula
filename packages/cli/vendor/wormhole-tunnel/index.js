"use strict";

async function connect(options) {
  const tenantId = options.tenantId || "default";
  const host = `https://webdav.tenant-${tenantId}.wormhole.internal`;
  return {
    host,
    async close() {}
  };
}

module.exports = {
  connect,
  createTunnel: connect,
  startTunnel: connect
};

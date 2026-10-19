/** Loads the original module in a mock factory. */
export async function load() {
  return await mocked<typeof import("./module")>();
}

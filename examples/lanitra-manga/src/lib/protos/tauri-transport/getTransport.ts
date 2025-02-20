export default async function getTransport() {
    return (await import("./index")).default;
}
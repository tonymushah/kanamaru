import { FeedsClient } from "$lib/protos/feeds.client";
import type { RpcTransport } from "@protobuf-ts/runtime-rpc";
import { createInfiniteQuery } from "@tanstack/svelte-query";

export function homeQuery(transport: RpcTransport) {
    const feedsClient = new FeedsClient(transport);
    return createInfiniteQuery({
        queryKey: ["home", "feed"],
        queryFn: async ({ pageParam }) => {
            return (await feedsClient.getHomeFeed({
                cursor: pageParam
            })).response;
        },
        initialPageParam: "",
        getNextPageParam: (lastPage) => lastPage.cursor
    })
}
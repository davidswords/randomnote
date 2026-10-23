import { accountsContract } from "./accounts.contract";
import { InMemoryAccounts } from "./in-memory-accounts";

accountsContract(() => new InMemoryAccounts());

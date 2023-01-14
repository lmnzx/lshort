import { useState } from "react";
import {
    InputGroup,
    Input,
    InputRightElement,
    Button,
    useToast,
} from "@chakra-ui/react";
import joi from "joi";

export default function CreateLink() {
    const api: string = import.meta.env.VITE_API ?? "http://localhost:3000";

    const [url, setUrl] = useState("");
    const toast = useToast();

    async function copyTextToClipboard(text: string) {
        if ("clipboard" in navigator) {
            return await navigator.clipboard.writeText(text);
        } else {
            return document.execCommand("copy", true, text);
        }
    }

    const sendIt = async (url: String) => {
        const validUrl = joi.string().uri();

        const validatedUrl = validUrl.validate(url);

        if (!validatedUrl.error) {
            const requestOptions = {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ value: url }),
            };

            const response = await fetch(`${api}/n`, requestOptions);
            const data = await response.json();
            if (data.error == null) {
                copyTextToClipboard(data.url);
                toast({
                    title: "Small Link Generated",
                    description: "The url is copied to your clipboard",
                    status: "success",
                    isClosable: true,
                });
            }
        } else {
            toast({
                title: "Invalid URL",
                description: "Make sure you copied the link properly",
                status: "error",
                isClosable: true,
            });
        }
    };

    return (
        <>
            <InputGroup size='md'>
                {/* <InputLeftAddon children='https://' /> */}
                <Input
                    pr='4.5rem'
                    placeholder='https://your-big-url.com'
                    onChange={(e) => setUrl(e.target.value)}
                />
                <InputRightElement width='6.45rem'>
                    <Button
                        h='1.75rem'
                        size='sm'
                        color='black'
                        bg='teal.100'
                        onClick={() => sendIt(url)}
                    >
                        Get Short
                    </Button>
                </InputRightElement>
            </InputGroup>
        </>
    );
}
